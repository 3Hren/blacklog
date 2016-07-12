use std::fmt::Arguments;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use {Config, Registry};

use filter::{Filter, FilterAction, NullFilter};
use handle::Handle;
use record::Record;

use factory::Factory;

/// Loggers are, well, responsible for logging. Nuff said.
pub trait Logger: Send + Sync {
    // TODO: Return a result, which can be ignored (without #[must_use]).
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>);
}

impl<T: Logger + ?Sized> Logger for Box<T> {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        self.deref().log(rec, args)
    }
}

// TODO: Implement.
// A logger wrapper that wraps other logger and filters incoming events by fast severity check.
// struct SeverityFilteredLoggerWrapper {}

// TODO: Docs.
/// # Note
///
/// The logger filter acts like a function to make filtering things common, but this may be
/// significant performance overhead for denied events, because to obtain a filter we mush lock a
/// mutex and copy a shared pointer containing the filter.
pub struct FilteredLoggerWrapper<L> {
    logger: L,
    filter: Arc<Mutex<Arc<Box<Filter>>>>,
}

impl<L: Logger> FilteredLoggerWrapper<L> {
    pub fn new(logger: L) -> FilteredLoggerWrapper<L> {
        FilteredLoggerWrapper {
            logger: logger,
            filter: Arc::new(Mutex::new(Arc::new(box NullFilter))),
        }
    }

    /// Replaces the current filter with the given one.
    pub fn filter(&self, filter: Box<Filter>) {
        *self.filter.lock().unwrap() = Arc::new(filter);
    }
}

impl<L: Logger> Logger for FilteredLoggerWrapper<L> {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        let filter = self.filter.lock().unwrap().clone();

        match filter.filter(&rec) {
            FilterAction::Deny => {}
            FilterAction::Accept | FilterAction::Neutral => {
                self.logger.log(rec, args)
            }
        }
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
    handlers: Arc<Vec<Box<Handle>>>,
}

impl SyncLogger {
    fn new(handlers: Vec<Box<Handle>>) -> SyncLogger {
        SyncLogger {
            handlers: Arc::new(handlers),
        }
    }
}

impl Logger for SyncLogger {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        // TODO: Maybe check whether a record was activated before.
        rec.activate(args);

        for handle in self.handlers.iter() {
            handle.handle(rec).unwrap();
        }
    }
}

impl Factory for SyncLogger {
    type Item = Logger;

    fn ty() -> &'static str {
        "synchronous"
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
    fn log_only_message() {
        let log = SyncLogger::new(vec![]);

        log!(log, 0, "file does not exist: /var/www/favicon.ico");
    }

    #[test]
    fn log_calls_handle() {
        struct MockHandle {
            counter: Arc<AtomicUsize>,
        }

        impl Handle for MockHandle {
            fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
                assert_eq!(0, rec.severity());
                self.counter.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let log = SyncLogger::new(vec![box MockHandle { counter: counter.clone() }]);

        log!(log, 0, "file does not exist: /var/www/favicon.ico");

        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

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
            fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
                self.counter.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let log = SyncLogger::new(vec![box MockHandle { counter: counter.clone() }]);
        let log = FilteredLoggerWrapper::new(log);

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
            box FilteredLoggerWrapper::new(log) as Box<Logger>
        }

        let log = box SyncLogger::new(vec![]);
        let log = create_wrapper(log);

        log!(log, 0, "");
    }
}

#[cfg(feature="benchmark")]
mod bench {
    use test::Bencher;

    use filter::FilterAction;
    use record::Record;
    use super::*;

    #[bench]
    fn log_message(b: &mut Bencher) {
        let log = SyncLogger::new(vec![]);

        b.iter(|| {
            log!(log, 0, "file does not exist: /var/www/favicon.ico");
        });
    }

    #[bench]
    fn log_message_with_format_and_meta6(b: &mut Bencher) {
        let log = SyncLogger::new(vec![]);

        b.iter(|| {
            log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
                flag: true,
                path1: "/home1",
                path2: "/home2",
                path3: "/home3",
                path4: "/home4",
                path5: "/home5",
            });
        });
    }

    #[bench]
    fn log_message_with_format_and_meta6_reject(b: &mut Bencher) {
        let log = SyncLogger::new(vec![]);
        let log = FilteredLoggerWrapper::new(log);
        log.filter(box |_rec: &Record| {
            FilterAction::Deny
        });

        b.iter(|| {
            log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
                flag: true,
                path1: "/home1",
                path2: "/home2",
                path3: "/home3",
                path4: "/home4",
                path5: "/home5",
            });
        });
    }

    #[bench]
    fn log_message_with_format_and_meta6_reject_fast(b: &mut Bencher) {
        use std::fmt::Arguments;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicIsize, Ordering};

        struct FastFilterLogger {
            sev: Arc<AtomicIsize>,
            wrapped: Arc<Box<Logger>>,
        }

        impl Logger for FastFilterLogger {
            fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
                if rec.severity() >= self.sev.load(Ordering::Relaxed) as i32 {
                    self.wrapped.log(rec, args);
                }
            }
        }

        let log = SyncLogger::new(vec![]);
        let log = FastFilterLogger {
            sev: Arc::new(AtomicIsize::new(1)),
            wrapped: Arc::new(box log),
        };

        b.iter(|| {
            log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
                flag: true,
                path1: "/home1",
                path2: "/home2",
                path3: "/home3",
                path4: "/home4",
                path5: "/home5",
            });
        });
    }
}
