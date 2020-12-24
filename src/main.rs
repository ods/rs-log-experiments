use std::{env, string::String};

use anyhow::Context; // For `context()`
use slog::Drain; // For `fuse()`

fn get_var(name: &str) -> Option<String> {
    env::var_os(name)
        .map(|value| value.into_string().unwrap())
        .filter(|value| !value.is_empty())
}

mod inner {
    pub fn log_in_inner() {
        slog_scope::info!("Hello from inner!");
    }
}

fn main() -> anyhow::Result<()> {
    // Pretend these vars came from outside
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("ENVIRONMENT", "dev");
    env::set_var("RUST_LOG", "info,log_experiments::inner=error");
    env::set_var("GRAYLOG_URL", "localhost:12201");
    env::set_var(
        "SENTRY_URL",
        "http://185b7a7e069f4ef0983c2467e79683b1@localhost:9001/1",
    );

    // Normal application flow starts here
    let environment =
        get_var("ENVIRONMENT").unwrap_or_else(|| "unknown".into());
    let options = logging::LoggingOptions {
        version: Some(env!("APP_VERSION").into()),
        filters: get_var("RUST_LOG"),
        environment: Some(environment.clone()),
        graylog: get_var("GRAYLOG_URL"),
        sentry: get_var("SENTRY_URL"),
    };
    let drain = logging::setup(options).context("Failed to setup logging")?;
    let logger = slog::Logger::root(
        drain.fuse(),
        slog::o!(
            "version" => env!("APP_VERSION"),
            "environment" => environment,
        ),
    );
    let log_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init()?;
    log_panics::init();

    slog_scope::info!("Hello, slog_scope!");
    slog_scope::error!("Scheiße");
    inner::log_in_inner();
    std::thread::spawn(|| panic!("Stirb!!!"))
        .join()
        .expect_err("Failed to panic");

    std::mem::drop(log_guard);
    Ok(())
}
