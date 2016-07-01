use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use {Config, InactiveRecord, Registry};

use handle::Handle;

use factory::Factory;

/// Loggers are, well, responsible for logging. Nuff said.
pub trait Logger: Send {
    // TODO: Return a result, which can be ignored (without #[must_use]).
    fn log<'a>(&self, record: &InactiveRecord<'a>);
}

#[derive(Clone)]
pub struct SyncLogger {
    severity: Arc<AtomicI32>,
    handlers: Arc<Vec<Box<Handle>>>,
}

impl SyncLogger {
    fn new(handlers: Vec<Box<Handle>>) -> SyncLogger {
        SyncLogger {
            severity: Arc::new(AtomicI32::new(0)),
            handlers: Arc::new(handlers),
        }
    }
}

impl Logger for SyncLogger {
    fn log<'a>(&self, rec: &InactiveRecord<'a>) {
        if rec.severity() >= self.severity.load(Ordering::Relaxed) {
            let mut rec = rec.activate();

            for handle in self.handlers.iter() {
                handle.handle(&mut rec).unwrap();
            }
        }
    }
}

pub struct SyncLoggerFactory;

impl Factory for SyncLoggerFactory {
    type Item = Logger;

    fn ty() -> &'static str {
        "synchronous"
    }

    fn from(&self, cfg: &Config, registry: &Registry) -> Result<Box<Logger>, Box<::std::error::Error>> {

        let iter = cfg.find("handlers")
            .ok_or("field \"handlers\" is required")?
            .as_array()
            .ok_or("field \"handlers\" must be an array")?
            .iter();

        let mut handlers = Vec::new();
        for handle in iter {
            println!("{:?}", handle);
            handlers.push(registry.handle(handle)?);
        }

        Ok(box SyncLogger::new(handlers))
    }
}

#[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {{
        $log.log(&$crate::Record::new($sev, line!(), module_path!(), format_args!($fmt, $($args)*),
            &$crate::MetaList::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ])
        ));
    }};
    ($log:ident, $sev:expr, $fmt:expr, {$($name:ident: $val:expr,)*}) => {{
        log!($log, $sev, $fmt, [], {$($name: $val,)*})
    }};
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*]) => {{
        log!($log, $sev, $fmt, [$($args)*], {})
    }};
    ($log:ident, $sev:expr, $fmt:expr, $($args:tt)*) => {{
        log!($log, $sev, $fmt, [$($args)*], {})
    }};
    ($log:ident, $sev:expr, $fmt:expr) => {{
        log!($log, $sev, $fmt, [], {})
    }};
);
