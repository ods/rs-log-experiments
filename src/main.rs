use lazy_static::lazy_static;
use regex::Regex;
use slog::{Drain, Fuse, Level};
use slog_async::Async;
use slog_gelf::Gelf;
use std::string::String;

struct DrainTee(Vec<Fuse<Async>>);

impl DrainTee {
    pub fn default() -> Self {
        Self(vec![])
    }

    pub fn push<D>(&mut self, drain: D)
    where
        D: Drain<Err = slog::Never, Ok = ()> + Send + 'static,
    {
        self.0.push(Async::default(drain).fuse());
    }

    pub fn term(mut self) -> anyhow::Result<Self> {
        let decorator = slog_term::TermDecorator::new().stderr().build();
        let term_drain = slog_term::FullFormat::new(decorator).build().fuse();
        self.push(term_drain);
        Ok(self)
    }

    pub fn graylog(mut self, url: &str) -> anyhow::Result<Self> {
        let host = hostname::get()?;
        let gelf_drain = Gelf::new(host.to_str().unwrap(), url)?.fuse();
        self.push(gelf_drain);
        Ok(self)
    }
}

impl Drain for DrainTee {
    type Ok = ();
    type Err = slog::Never;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        self.0
            .iter()
            .try_for_each(|drain| drain.log(record, values).map(|_| ()))
    }
}

lazy_static! {
    // The resulting format is `major.minor.patch[+N+hash][+dirty]`.  The parts
    // are:
    //   * release (major.minor.patch)
    //   * number of commits `N` on top of released version in development
    //     version with commit hash abbreviation `hash`
    //   * `dirty` marker for local uncommitted changes
    //
    // The release and revision come from annotated tag matching "*.*.*"
    // pattern.  It falls back on `hash[+dirty]` when no release tag yet.
    //
    // `--match='*.*.*'` to take into account release tags only (and only
    //      annotated, not lightweight)
    // `--always` is for fallback while there were no releases yet
    // `--first-parent` discards merge with revision tag after release (beware
    //      that this might become undesired if we switch to git-flow)
    // `--dirty` adds "-dirty" suffix when there are uncommited changes
    //
    // "-" are replaced with "+" since RPM doesn't allow dashes in version
    // ("." is not used to avoid collision with revision).
    // "g" (stands for "git") before hash is removed to avoid
    // misinterpretation.
    static ref APP_VERSION: String = Regex::new("-(?P<n_commits>[0-9]+)-g")
        .unwrap()
        .replace(
            git_version::git_version!(
                args = [
                    "--always",
                    "--match=*.*.*",
                    "--first-parent",
                    "--dirty"
                ]
            ),
            "+$n_commits+",
        )
        .to_string();
}

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("GRAYLOG_URL", "localhost:12201");

    let graylog_url = std::env::var("GRAYLOG_URL")?;
    let drain_tee = DrainTee::default().term()?.graylog(&graylog_url)?;

    let logger = slog::Logger::root(
        drain_tee.filter_level(Level::Info).fuse(),
        slog::o!("version" => &*APP_VERSION),
    );
    let log_guard = slog_scope::set_global_logger(logger);

    slog_scope::error!("Hello, slog_scope!");

    std::mem::drop(log_guard);
    Ok(())
}
