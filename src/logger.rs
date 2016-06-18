use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use {Record, Severity};

pub struct Logger {
    severity: AtomicIsize,
    tx: mpsc::Sender<()>, // TODO: RecordBuf.
    thread: JoinHandle<()>,
}

impl Logger {
    pub fn new() -> Logger {
        let (tx, rx) = mpsc::channel();

        let thread = thread::spawn(move || {
            for event in rx {
            }
        });

        Logger {
            severity: AtomicIsize::new(0),
            tx: tx,
            thread: thread,
        }
    }

    pub fn log(&self, severity: Severity, message: &str) {
        if severity <= self.severity.load(Ordering::Relaxed) {
            let rec = Record::new(severity, message);
            self.tx.send(()).unwrap();
        }
    }
}

// #[macro_export]
// macro_rules! log(
//     ($log:expr, $severity:expr, $fmt:expr, ($($arg:tt)*), {$($name:tt: $value:tt)*}) => {
//     }
// );
