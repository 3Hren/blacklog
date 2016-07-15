#[macro_use] extern crate blacklog;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use blacklog::{Handle, Logger, Record};
use blacklog::logger::SyncLogger;

#[test]
fn log_only_message() {
    let log = SyncLogger::new(vec![]);

    log!(log, 0, "file does not exist: /var/www/favicon.ico");
}

struct MockHandle {
    counter: Arc<AtomicUsize>,
}

impl MockHandle {
    fn new() -> MockHandle {
        MockHandle {
            counter: Arc::new(AtomicUsize::new(0))
        }
    }

    fn counter(&self) -> Arc<AtomicUsize> {
        self.counter.clone()
    }
}

impl Handle for MockHandle {
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
        assert_eq!(0, rec.severity());
        self.counter.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }
}

#[test]
fn log_calls_handle() {
    let handle = MockHandle::new();
    let counter = handle.counter();
    let log = SyncLogger::new(vec![Box::new(handle)]);

    log!(log, 0, "file does not exist: /var/www/favicon.ico");

    assert_eq!(1, counter.load(Ordering::SeqCst));
}

#[test]
fn log_calls_handle_after_clone() {
    let handle = MockHandle::new();
    let counter = handle.counter();
    let log1 = SyncLogger::new(vec![Box::new(handle)]);
    let log2 = log1.clone();

    log!(log1, 0, "file does not exist: /var/www/favicon.ico");
    assert_eq!(1, counter.load(Ordering::SeqCst));

    log!(log2, 0, "file does not exist: /var/www/favicon.ico");
    assert_eq!(2, counter.load(Ordering::SeqCst));
}

#[test]
fn log_calls_handle_after_reset() {
    let handle = MockHandle::new();
    let counter = handle.counter();
    let log1 = SyncLogger::new(vec![]);
    let log2 = log1.clone();

    log2.reset(vec![Box::new(handle)]);

    log!(log1, 0, "file does not exist: /var/www/favicon.ico");
    assert_eq!(1, counter.load(Ordering::SeqCst));

    log!(log2, 0, "file does not exist: /var/www/favicon.ico");
    assert_eq!(2, counter.load(Ordering::SeqCst));
}

// #[test]
// fn log_macro_use() {
//     let log = SyncLogger::new(vec![]);
//
//     // Add some meta information.
//     log!(log, 0, "file does not exist: /var/www/favicon.ico", {
//         path: "/home",
//     });
//
//     // Delayed formatting.
//     log!(log, 0, "file does not exist: {}", "/var/www/favicon.ico");
//
//     // Alternative syntax for delayed formatting without additional meta information.
//     log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"]);
//
//     // Full syntax both with delayed formatting and meta information.
//     log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
//         flag: true,
//         path: "/home",
//         path: "/home/esafronov", // Duplicates are allowed as a stacking feature.
//         target: "core",
//         owned: "le message".to_string(),
//     });
// }

// #[test]
// fn log_fn() {
//     let log = SyncLogger::new(vec![]);
//
//     fn fact(n: u64) -> u64 {
//         match n {
//             0 | 1 => 1,
//             n => n * fact(n - 1),
//         }
//     };
//
//     let val = true;
//
//     // Only severity, message and meta information.
//     log!(log, 0, "file does not exist: /var/www/favicon.ico", {
//         lazy: FnMeta::new(move || { format!("lazy message of {}", val) }),
//         lazy: FnMeta::new(move || val ),
//         lazy: FnMeta::new(move || fact(10)),
//     });
// }
//
// #[test]
// fn log_filter_by_severity() {
//     let handle = MockHandle::new();
//     let counter = handle.counter();
//     let log = SyncLogger::new(vec![Box::new(handle)]);
//     let log = FilteredLoggerAdapter::new(log);
//
//     log.filter(Box::new(|rec: &Record| {
//         if rec.severity() >= 1 {
//             FilterAction::Neutral
//         } else {
//             FilterAction::Deny
//         }
//     }));
//
//     log!(log, 0, "");
//     assert_eq!(0, counter.load(Ordering::SeqCst));
//     log!(log, 1, "");
//     assert_eq!(1, counter.load(Ordering::SeqCst));
// }
//
// #[test]
// fn log_filter_box() {
//     fn create_wrapper(log: Box<Logger>) -> Box<Logger> {
//         Box::new(FilteredLoggerAdapter::new(log)) as Box<Logger>
//     }
//
//     let log = Box::new(SyncLogger::new(vec![]));
//     let log = create_wrapper(log);
//
//     log!(log, 0, "");
// }
