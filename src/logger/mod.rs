use std::fmt::Arguments;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use {Config, Registry};

use handle::Handle;
use record::Record;
use factory::Factory;

pub use self::actor::ActorLogger;
// pub use self::sync::SyncLogger;
pub use self::filtered::{FilteredLoggerAdapter, SeverityFilteredLoggerAdapter};

mod actor;
mod filtered;
mod sync;

/// Loggers are, well, responsible for logging. Nuff said.
pub trait Logger: Send {
    // TODO: Return a result, which can be ignored (without #[must_use]).
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>);
}

impl<T: Logger + ?Sized> Logger for Box<T> {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        self.deref().log(rec, args)
    }
}

/// Blocking, but still fast, thread-safe reloadable synchronous logger.
///
/// Represents a logger, which handles incoming records by sequentially iterating through the given
/// handlers.
///
/// Such kind of logger is required to implement zero-copy meta information handling, through its
/// borrowing without prior converting into owned structures. In these cases it's strongly
/// recommended that all of handlers and outputs won't block no matter what. For example UDP output
/// perfectly fits in this recommendation, but the File output - does not, which may results in
/// threads freezing in the case hardware I/O problems.
///
/// This logger acts like a root logger - the base of other functionality like filtering, which can
/// be provided by wrapping instances of this struct.
///
/// By reloading we mean that this logger can be safely reassigned in runtime, allowing both to
/// change configuration and to correctly finish all outstanding operations, like flushing. This
/// feature gives an ability to implement popular SIGHUP logging rotation.
#[derive(Clone)]
pub struct SyncLogger {
    handlers: Arc<Mutex<Arc<Vec<Box<Handle>>>>>,
}

impl SyncLogger {
    pub fn new(handlers: Vec<Box<Handle>>) -> SyncLogger {
        SyncLogger {
            handlers: Arc::new(Mutex::new(Arc::new(handlers))),
        }
    }

    pub fn reset(&self, handlers: Vec<Box<Handle>>) {
        *self.handlers.lock().unwrap() = Arc::new(handlers);
    }
}

impl Logger for SyncLogger {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        // TODO: Maybe check whether a record was activated before.
        rec.activate(args);

        let handlers = self.handlers.lock().unwrap();
        for handle in handlers.iter() {
            handle.handle(rec).unwrap();
        }
    }
}

impl Factory for SyncLogger {
    type Item = Logger;

    fn ty() -> &'static str {
        "sync"
    }

    fn from(cfg: &Config, registry: &Registry) -> Result<Box<Logger>, Box<::std::error::Error>> {
        let handlers = cfg.find("handlers")
            .ok_or("field \"handlers\" is required")?
            .as_array()
            .ok_or("field \"handlers\" must be an array")?
            .iter()
            .map(|cfg| registry.handle(cfg))
            .collect()?;

        let res = box SyncLogger::new(handlers);

        Ok(res)
    }
}

// TODO: Docs.
#[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {{
        $log.log(&mut $crate::Record::new($sev, line!(), module_path!(),
            &$crate::MetaLink::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ])
        ), format_args!($fmt, $($args)*));
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use {Handle, FnMeta, Record};
    use filter::FilterAction;
    use super::*;

    #[test]
    fn log_macro_use() {
        let log = SyncLogger::new(vec![]);

        // Add some meta information.
        log!(log, 0, "file does not exist: /var/www/favicon.ico", {
            path: "/home",
        });

        // Delayed formatting.
        log!(log, 0, "file does not exist: {}", "/var/www/favicon.ico");

        // Alternative syntax for delayed formatting without additional meta information.
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"]);

        // Full syntax both with delayed formatting and meta information.
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
            flag: true,
            path: "/home",
            path: "/home/esafronov", // Duplicates are allowed as a stacking feature.
            target: "core",
            owned: "le message".to_string(),
        });
    }

    #[test]
    fn log_fn() {
        let log = SyncLogger::new(vec![]);

        fn fact(n: u64) -> u64 {
            match n {
                0 | 1 => 1,
                n => n * fact(n - 1),
            }
        };

        let val = true;

        // Only severity, message and meta information.
        log!(log, 0, "file does not exist: /var/www/favicon.ico", {
            lazy: FnMeta::new(move || { format!("lazy message of {}", val) }),
            lazy: FnMeta::new(move || val ),
            lazy: FnMeta::new(move || fact(10)),
        });
    }

    #[test]
    fn log_filter_by_severity() {
        struct MockHandle {
            counter: Arc<AtomicUsize>,
        }

        impl Handle for MockHandle {
            fn handle(&self, _rec: &mut Record) -> Result<(), ::std::io::Error> {
                self.counter.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let log = SyncLogger::new(vec![box MockHandle { counter: counter.clone() }]);
        let log = FilteredLoggerAdapter::new(log);

        log.filter(box |rec: &Record| {
            if rec.severity() >= 1 {
                FilterAction::Neutral
            } else {
                FilterAction::Deny
            }
        });

        log!(log, 0, "");
        assert_eq!(0, counter.load(Ordering::SeqCst));
        log!(log, 1, "");
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[test]
    fn log_filter_box() {
        fn create_wrapper(log: Box<Logger>) -> Box<Logger> {
            box FilteredLoggerAdapter::new(log) as Box<Logger>
        }

        let log = box SyncLogger::new(vec![]);
        let log = create_wrapper(log);

        log!(log, 0, "");
    }
}
