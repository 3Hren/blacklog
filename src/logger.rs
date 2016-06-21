use std::fmt::Arguments;
use std::io::Write;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use {Meta, MetaList, Record, Severity};

pub type Error = ::std::io::Error;

pub struct Logger {
    severity: AtomicIsize,
    tx: mpsc::Sender<()>, // TODO: <RecordBuf>.
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

    // TODO:
    // For asynchronous logger:
    // - Pass severity filtering.
    // - Format.
    // - Make RecordBuf.
    // - Pass to the channel.
    // - Pass custom filtering.
    // - Layout.
    // - Broadcast to appenders.
    // For synchronous logger:
    // - Pass severity filtering.
    // - Pass custom filtering.
    // - Format.
    // - Make Record.
    // - Layout.
    // - Broadcast to appenders.
    fn log<'a>(&self, sev: Severity, format: Arguments<'a>, meta: &MetaList<'a>) {
        if sev >= self.severity.load(Ordering::Relaxed) {
            // Do magic.
        }
    }
}

#[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {
        $log.log($sev, format_args!($fmt, $($args)*), &MetaList::new(
            &[$(Meta::new(stringify!($name), $val)),*]
        ));
    };
    ($log:ident, $sev:expr, $fmt:expr, {$($name:ident: $val:expr,)*}) => {
        log!($log, $sev, $fmt, [], {$($name: $val,)*})
    };
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*]) => {
        $log.log($sev, format_args!($fmt, $($args)*), &MetaList::new(&[]));
    };
    ($log:ident, $sev:expr, $fmt:expr, $($args:tt)*) => {
        $log.log($sev, format_args!($fmt, $($args)*), &MetaList::new(&[]));
    };
    ($log:ident, $sev:expr, $fmt:expr) => {
        $log.log($sev, format_args!($fmt), &MetaList::new(&[]));
    };
);

#[cfg(test)]
mod tests {
    use {Meta, MetaList};
    use super::{Logger};

    #[test]
    fn log() {
        let log = Logger::new();

        // Only severity with message.
        log!(log, 0, "file does not exist: /var/www/favicon.ico");

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
            target: "core",
            owned: "le message".to_string(),
        });

        // Ideal:
        // log!(logger, 0, "le message: {name}, {}", 42,
        //     name: "Vasya",
        //     path: "/usr/bin"
        // );
    }
}
