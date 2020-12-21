use std::any::Any;
use std::boxed::Box;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::string::String;

use sentry_slog::SentryDrain;
use slog::{Drain, Fuse};
use slog_async::Async;
use slog_gelf::Gelf;

pub struct DrainTee {
    drains: Vec<Fuse<Async>>,
    guards: Vec<Box<dyn Any + Send + Sync + RefUnwindSafe + UnwindSafe>>,
    version: Option<String>,
    environment: Option<String>,
}

impl DrainTee {
    pub fn default() -> Self {
        Self {
            drains: vec![],
            guards: vec![],
            version: None,
            environment: None,
        }
    }

    pub fn version(mut self, value: &str) -> Self {
        self.version = Some(value.into());
        self
    }

    pub fn environment(mut self, value: &str) -> Self {
        self.environment = Some(value.into());
        self
    }

    pub fn push<D>(&mut self, drain: D)
    where
        D: Drain<Err = slog::Never, Ok = ()> + Send + 'static,
    {
        self.drains.push(Async::default(drain).fuse());
    }

    pub fn term(mut self) -> anyhow::Result<Self> {
        let decorator = slog_term::TermDecorator::new().stderr().build();
        let term_drain = slog_term::FullFormat::new(decorator).build().fuse();
        self.push(term_drain);
        Ok(self)
    }

    pub fn graylog(mut self, url: &str) -> anyhow::Result<Self> {
        let host = hostname::get()?;
        let drain = Gelf::new(host.to_str().unwrap(), url)?.fuse();
        self.push(drain);
        Ok(self)
    }

    pub fn sentry(mut self, url: &str) -> anyhow::Result<Self> {
        let dsn = url.parse()?;
        let sentry = sentry::init(sentry::ClientOptions {
            dsn: Some(dsn),
            // FIXME version and environment can be set after `sentry()` call
            release: self.version.clone().map(Into::into),
            environment: self.environment.clone().map(Into::into),
            max_breadcrumbs: 0,
            ..Default::default()
        });
        self.guards.push(Box::new(sentry));
        let drain = SentryDrain::new(slog::Discard);
        self.push(drain);
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
        self.drains
            .iter()
            .try_for_each(|drain| drain.log(record, values).map(|_| ()))
    }
}
