use std::panic::{RefUnwindSafe, UnwindSafe};
use std::string::String;

use sentry_slog::SentryDrain;
use slog::{Drain, Fuse};
use slog_async::Async;
use slog_gelf::Gelf;

/// Wrap drain along with guard to be dropped when wrapper is dropped
pub struct DrainWithGuard<D, G>
where
    D: Drain,
    G: Send + Sync + RefUnwindSafe + UnwindSafe,
{
    pub drain: D,
    #[allow(dead_code)]
    pub guard: G,
}

impl<D, G> Drain for DrainWithGuard<D, G>
where
    D: Drain,
    G: Send + Sync + RefUnwindSafe + UnwindSafe,
{
    type Ok = D::Ok;
    type Err = D::Err;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        self.drain.log(record, values)
    }
}

/// Drain that duplicates log record into several subdrains
#[derive(Default)]
pub struct DrainTee {
    drains: Vec<Fuse<Async>>,
}

impl DrainTee {
    fn push<D>(&mut self, drain: D)
    where
        D: Drain<Err = slog::Never, Ok = ()> + Send + 'static,
    {
        self.drains.push(Async::default(drain).fuse());
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
        self.drains
            .iter()
            .try_for_each(|drain| drain.log(record, values).map(|_| ()))
    }
}

#[derive(Default)]
pub struct LoggingOptions {
    pub version: Option<String>,
    pub environment: Option<String>,
    pub graylog: Option<String>,
    pub sentry: Option<String>,
}

pub fn setup(options: LoggingOptions) -> anyhow::Result<DrainTee> {
    let mut tee = DrainTee::default();

    tee.push(
        slog_term::FullFormat::new(
            slog_term::TermDecorator::new().stderr().build(),
        )
        .build()
        .fuse(),
    );

    if let Some(graylog_url) = options.graylog {
        tee.push(
            Gelf::new(hostname::get()?.to_str().unwrap(), &graylog_url)?.fuse(),
        );
    }

    if let Some(sentry_url) = options.sentry {
        let sentry = sentry::init(sentry::ClientOptions {
            dsn: Some(sentry_url.parse()?),
            release: options.version.clone().map(Into::into),
            environment: options.environment.clone().map(Into::into),
            max_breadcrumbs: 0,
            ..Default::default()
        });
        tee.push(DrainWithGuard {
            drain: SentryDrain::new(slog::Discard),
            guard: sentry,
        });
    }

    Ok(tee)
}
