use slog::{Drain, Fuse, Level};
use slog_async::Async;
use slog_gelf::Gelf;

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

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("GRAYLOG_URL", "localhost:12201");

    let graylog_url = std::env::var("GRAYLOG_URL")?;
    let drain_tee = DrainTee::default().term()?.graylog(&graylog_url)?;

    let logger = slog::Logger::root(
        drain_tee.filter_level(Level::Info).fuse(),
        slog::o!(),
    );
    let log_guard = slog_scope::set_global_logger(logger);

    slog_scope::error!("Hello, slog_scope!");

    std::mem::drop(log_guard);
    Ok(())
}
