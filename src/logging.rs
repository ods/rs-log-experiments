use std::any::Any;
use std::boxed::Box;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::string::String;

use sentry_slog::SentryDrain;
use slog::{Drain, Fuse};
use slog_async::Async;
use slog_gelf::Gelf;

#[derive(Default)]
pub struct DrainTeeOptions {
    pub version: Option<String>,
    pub environment: Option<String>,
    pub graylog: Option<String>,
    pub sentry: Option<String>,
}

pub struct DrainTee {
    version: Option<String>,
    environment: Option<String>,
    drains: Vec<Fuse<Async>>,
    guards: Vec<Box<dyn Any + Send + Sync + RefUnwindSafe + UnwindSafe>>,
}

impl DrainTee {
    pub fn new(options: DrainTeeOptions) -> anyhow::Result<Self> {
        let mut drain = Self {
            version: options.version,
            environment: options.environment,
            drains: vec![],
            guards: vec![],
        };
        drain.term()?;
        if let Some(graylog_url) = options.graylog {
            drain.graylog(&graylog_url)?;
        }
        if let Some(sentry_url) = options.sentry {
            drain.sentry(&sentry_url)?;
        }
        Ok(drain)
    }

    fn push<D>(&mut self, drain: D)
    where
        D: Drain<Err = slog::Never, Ok = ()> + Send + 'static,
    {
        self.drains.push(Async::default(drain).fuse());
    }

    fn term(&mut self) -> anyhow::Result<()> {
        let decorator = slog_term::TermDecorator::new().stderr().build();
        let term_drain = slog_term::FullFormat::new(decorator).build().fuse();
        self.push(term_drain);
        Ok(())
    }

    fn graylog(&mut self, url: &str) -> anyhow::Result<()> {
        let host = hostname::get()?;
        let drain = Gelf::new(host.to_str().unwrap(), url)?.fuse();
        self.push(drain);
        Ok(())
    }

    fn sentry(&mut self, url: &str) -> anyhow::Result<()> {
        let dsn = url.parse()?;
        let sentry = sentry::init(sentry::ClientOptions {
            dsn: Some(dsn),
            release: self.version.clone().map(Into::into),
            environment: self.environment.clone().map(Into::into),
            max_breadcrumbs: 0,
            ..Default::default()
        });
        self.guards.push(Box::new(sentry));
        let drain = SentryDrain::new(slog::Discard);
        self.push(drain);
        Ok(())
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
