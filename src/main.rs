use slog::Drain;
use slog_async::Async;
use slog_gelf::Gelf;

const LOG_BUFFER_SIZE: usize = 128;

#[derive(Debug)]
struct DrainMux<D>(Vec<D>);

impl<D> DrainMux<D> {
    pub fn new(d: Vec<D>) -> Self {
        Self(d)
    }
}

impl<D: Drain> Drain for DrainMux<D> {
    type Ok = ();
    type Err = D::Err;

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

    let decorator = slog_term::TermDecorator::new().stderr().build();
    let term_drain = slog_term::FullFormat::new(decorator).build().fuse();

    let graylog_url = std::env::var("GRAYLOG_URL")?;
    let host = hostname::get()?;
    let gelf_drain = Gelf::new(host.to_str().unwrap(), &graylog_url)?.fuse();

    let drain_mux = DrainMux::new(vec![
        Async::new(term_drain)
            .chan_size(LOG_BUFFER_SIZE)
            .build()
            .fuse(),
        Async::new(gelf_drain)
            .chan_size(LOG_BUFFER_SIZE)
            .build()
            .fuse(),
    ])
    .fuse();
    let logger = slog::Logger::root(drain_mux, slog::o!());
    let log_guard = slog_scope::set_global_logger(logger);
    // slog::error!(logger, "Hello, slog!");
    slog_scope::error!("Hello, slog_scope/drain_mux!");
    log::error!("Hello, log!");
    println!("Hello, world!");
    std::mem::drop(log_guard);
    Ok(())
}
