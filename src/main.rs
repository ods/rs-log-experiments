use std::env;

use anyhow::Context; // For `context()`

pub(crate) mod environ;
pub(crate) mod logging;

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
    let log_guard = logging::setup_from_env(Some(env!("APP_VERSION")))
        .context("Failed to setup logging")?;

    slog_scope::info!("Hello, slog_scope!");
    slog_scope::error!("Scheiße");
    inner::log_in_inner();
    std::thread::spawn(|| panic!("Stirb!!!"))
        .join()
        .expect_err("Failed to panic");

    std::mem::drop(log_guard);
    Ok(())
}
