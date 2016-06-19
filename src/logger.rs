use std::fmt::Arguments;
use std::io::Write;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use {Record, Severity};

pub enum Value<'a> {
    Nil,
    // Bool(bool),
    // Signed(i64),
    // Unsigned(u64),
    // Float(f64),
    String(&'a str),
    // Func(&'a Fn(&mut Write) -> Result<(), ::std::io::Error>),
}

impl<'a> Into<Value<'a>> for &'a str {
    fn into(self) -> Value<'a> {
        Value::String(&self)
    }
}

pub struct Meta<'a> {
    name: &'a str,
    value: Value<'a>,
}

impl<'a> Meta<'a> {
    pub fn new<V>(name: &'a str, value: V) -> Meta<'a>
        where V: Into<Value<'a>>
    {
        Meta {
            name: name,
            value: value.into(),
        }
    }
}

pub struct MetaList<'a> {
    prev: Option<&'a MetaList<'a>>,
    meta: &'a [Meta<'a>],
}

impl<'a> MetaList<'a> {
    pub fn new(meta: &'a [Meta<'a>]) -> MetaList<'a> {
        MetaList::next(meta, None)
    }

    pub fn next(meta: &'a [Meta<'a>], prev: Option<&'a MetaList<'a>>) -> MetaList<'a> {
        MetaList {
            prev: prev,
            meta: meta,
        }
    }
}

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
    fn log<'a>(&self, sev: Severity, format: Arguments<'a>, meta: &MetaList<'a>) ->
        Result<(), Error>
    {
        if sev >= self.severity.load(Ordering::Relaxed) {
            // Do magic.
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {
        $log.log_($sev, format_args!($fmt, $($args)*), &MetaList::new(
            &[$(Meta::new(stringify!($name), $val)),*]
        ));
    };
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*]) => {
        $log.log_($sev, format_args!($fmt, $($args)*), &MetaList::new(&[]));
    };
    ($log:ident, $sev:expr, $fmt:expr, $($args:tt)*) => {
        $log.log_($sev, format_args!($fmt, $($args)*), &MetaList::new(&[]));
    };
    ($log:ident, $sev:expr, $fmt:expr) => {
        $log.log_($sev, format_args!($fmt), &MetaList::new(&[]));
    };
);

#[cfg(test)]
mod tests {
    use super::{Logger, Meta, MetaList};

    #[test]
    fn log() {
        let log = Logger::new();

        log.log(0, "le message", &MetaList::new(&[
            Meta::new("path", "/usr/bin/env"),
        ]));

        log!(log, 0, "file does not exist: /var/www/favicon.ico");
        log!(log, 0, "file does not exist: {}", "/var/www/favicon.ico");
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"]);
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
            path: "/home",
            target: "core",
        });

        // Ideal:
        // log!(logger, 0, "le message: {name}, {}", 42,
        //     name: "Vasya",
        //     path: "/usr/bin"
        // );
    }
}
