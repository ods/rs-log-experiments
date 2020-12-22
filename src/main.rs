use slog::Drain;

use logging::DrainTee;

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("ENVIRONMENT", "dev");
    std::env::set_var("GRAYLOG_URL", "localhost:12201");
    std::env::set_var(
        "SENTRY_URL",
        "http://185b7a7e069f4ef0983c2467e79683b1@localhost:9001/1",
    );

    let environment = std::env::var("ENVIRONMENT")?;
    let graylog_url = std::env::var("GRAYLOG_URL")?;
    let sentry_url = std::env::var("SENTRY_URL")?;
    let drain_tee = DrainTee::new(env!("APP_VERSION"), &environment)
        .term()?
        .graylog(&graylog_url)?
        .sentry(&sentry_url)?;

    let logger = slog::Logger::root(
        drain_tee.filter_level(slog::Level::Info).fuse(),
        slog::o!("version" => env!("APP_VERSION")),
    );
    let log_guard = slog_scope::set_global_logger(logger);

    slog_scope::error!("Hello, slog_scope!");

    std::mem::drop(log_guard);
    Ok(())
}
