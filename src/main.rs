use slog::Drain;

use logging::DrainTee;

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("GRAYLOG_URL", "localhost:12201");

    let graylog_url = std::env::var("GRAYLOG_URL")?;
    let drain_tee = DrainTee::default().term()?.graylog(&graylog_url)?;

    let logger = slog::Logger::root(
        drain_tee.filter_level(slog::Level::Info).fuse(),
        slog::o!("version" => env!("APP_VERSION")),
    );
    let log_guard = slog_scope::set_global_logger(logger);

    slog_scope::error!("Hello, slog_scope!");

    std::mem::drop(log_guard);
    Ok(())
}
