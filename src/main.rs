use std::string::String;

use slog::Drain; // Needed for `filter_level()` and `fuse()`

use logging::DrainTee;

fn get_var(name: &str) -> Option<String> {
    std::env::var_os(name)
        .map(|value| value.into_string().unwrap())
        .filter(|value| !value.is_empty())
}

fn main() -> anyhow::Result<()> {
    // Pretend these vars came from outside
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("ENVIRONMENT", "dev");
    std::env::set_var("GRAYLOG_URL", "localhost:12201");
    std::env::set_var(
        "SENTRY_URL",
        "http://185b7a7e069f4ef0983c2467e79683b1@localhost:9001/1",
    );

    // Normal application flow starts here
    let environment =
        get_var("ENVIRONMENT").unwrap_or_else(|| "unknown".into());
    let mut drain_tee =
        DrainTee::new(env!("APP_VERSION"), &environment).term()?;
    if let Some(graylog_url) = get_var("GRAYLOG_URL") {
        drain_tee = drain_tee.graylog(&graylog_url)?
    }
    if let Some(sentry_url) = get_var("SENTRY_URL") {
        drain_tee = drain_tee.sentry(&sentry_url)?;
    }
    let logger = slog::Logger::root(
        drain_tee.filter_level(slog::Level::Info).fuse(),
        slog::o!(
            "version" => env!("APP_VERSION"),
            "environment" => environment,
        ),
    );
    let log_guard = slog_scope::set_global_logger(logger);

    slog_scope::error!("Hello, slog_scope!");

    std::mem::drop(log_guard);
    Ok(())
}
