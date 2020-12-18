use slog::{Drain, Fuse};
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

    let mut drain_tee = DrainTee::default();

    let decorator = slog_term::TermDecorator::new().stderr().build();
    let term_drain = slog_term::FullFormat::new(decorator).build().fuse();
    drain_tee.push(term_drain);

    let graylog_url = std::env::var("GRAYLOG_URL")?;
    let host = hostname::get()?;
    let gelf_drain = Gelf::new(host.to_str().unwrap(), &graylog_url)?.fuse();
    drain_tee.push(gelf_drain);

    let logger = slog::Logger::root(drain_tee, slog::o!());
    let log_guard = slog_scope::set_global_logger(logger);

    slog_scope::error!("Hello, slog_scope!");

    std::mem::drop(log_guard);
    Ok(())
}
