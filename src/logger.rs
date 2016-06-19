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

    // fn log<'a, S>(&self, severity: S, format: Arguments<'a>, meta: &MetaList<'a>) -> Result<(), Error>
    //     where S: Severity;
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

    pub fn log<'a>(&self, severity: Severity, message: &str, meta: &MetaList<'a>) {
        if severity >= self.severity.load(Ordering::Relaxed) {
            let rec = Record::new(severity, message);
            self.tx.send(()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Logger, Meta, MetaList};

    #[test]
    fn log() {
        let log = Logger::new();

        log.log(0, "le message", &MetaList::new(&[
            Meta::new("path", "/usr/bin/env"),
        ]));

        // Ideal:
        // log!(logger, 0, "le message: {name}, {}", 42,
        //     name: "Vasya",
        //     path: "/usr/bin"
        // );
        // -> Arguments<'a>, &'a [Meta<'a>].
    }
}
