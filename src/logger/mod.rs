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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use {Handle, FnMeta, Record};
    use super::*;

    #[test]
    fn log_only_message() {
        let log = SyncLogger::new(vec![]);

        log!(log, 0, "file does not exist: /var/www/favicon.ico");
    }

    #[test]
    fn log_calls_handle() {
        struct MockHandle {
            flag: Arc<AtomicBool>,
        }

        impl Handle for MockHandle {
            fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
                assert_eq!(0, rec.severity());
                self.flag.store(true, Ordering::SeqCst);

                Ok(())
            }
        }

        let flag = Arc::new(AtomicBool::new(false));
        let log = SyncLogger::new(vec![box MockHandle { flag: flag.clone() }]);

        log!(log, 0, "file does not exist: /var/www/favicon.ico");

        assert_eq!(true, flag.load(Ordering::SeqCst));
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
}

#[cfg(feature="benchmark")]
mod bench {
    use test::Bencher;

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

        b.iter(|| {
            log!(log, -1, "file does not exist: {}", ["/var/www/favicon.ico"], {
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
