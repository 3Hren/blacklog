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
