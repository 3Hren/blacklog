use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};

use {Config, InactiveRecord, Registry};

use factory::Factory;

/// Loggers are, well, responsible for logging. Nuff said.
pub trait Logger: Send {
    // TODO: Return a result, which can be ignored (without #[must_use]).
    fn log<'a>(&self, record: &InactiveRecord<'a>);
}

#[derive(Clone)]
pub struct SyncLogger;

impl Logger for SyncLogger {
    fn log<'a>(&self, record: &InactiveRecord<'a>) {
        unimplemented!();
    }
}

pub struct SyncLoggerFactory;

impl Factory for SyncLoggerFactory {
    type Item = Logger;

    fn ty() -> &'static str {
        "synchronous"
    }

    fn from(&self, _cfg: &Config, _registry: &Registry) -> Result<Box<Logger>, Box<::std::error::Error>> {
        Ok(box SyncLogger)
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
