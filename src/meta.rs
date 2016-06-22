use std::borrow::Cow;
use std::fmt::{Arguments, Debug};
use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::thread::{self, JoinHandle};

use chrono::{DateTime, UTC};

use Severity;

pub type Error = ::std::io::Error;

#[derive(Debug, Copy, Clone)]
pub struct Meta<'a> {
    name: &'static str,
    value: &'a Encode,
}

impl<'a> Meta<'a> {
    pub fn new(name: &'static str, value: &'a Encode) -> Meta<'a> {
        Meta {
            name: name,
            value: value,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct MetaBuf {
    name: &'static str,
    value: Box<EncodeBuf>,
}

impl MetaBuf {
    fn new(name: &'static str, value: Box<EncodeBuf>) -> MetaBuf {
        MetaBuf {
            name: name,
            value: value,
        }
    }
}

impl<'a> From<&'a MetaList<'a>> for Vec<MetaBuf> {
    fn from(val: &'a MetaList<'a>) -> Vec<MetaBuf> {
        let mut result = Vec::with_capacity(32);

        let mut node = val;
        loop {
            for meta in node.meta.iter().rev() {
                result.push(MetaBuf::new(meta.name, meta.value.to_encode_buf()));
            }

            if let Some(prev) = node.prev {
                node = prev;
            } else {
                break;
            }
        }

        result
    }
}

pub trait Encode : Debug + Send + Sync {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error>;

    fn to_encode_buf(&self) -> Box<EncodeBuf>;
}

pub trait EncodeBuf : Encode {}

impl<T: Encode> EncodeBuf for T {}

pub trait Encoder {
    fn encode_bool(&mut self, value: bool) -> Result<(), Error>;
    fn encode_str(&mut self, value: &str) -> Result<(), Error>;
}

impl Encode for bool {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_bool(*self)
    }

    fn to_encode_buf(&self) -> Box<EncodeBuf> {
        box self.to_owned()
    }
}

impl Encode for &'static str {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(self)
    }

    fn to_encode_buf(&self) -> Box<EncodeBuf> {
        // box self.to_owned()
        box Cow::Borrowed(*self)
    }
}

impl Encode for str {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(self)
    }

    fn to_encode_buf(&self) -> Box<EncodeBuf> {
        box self.to_owned()
    }
}

impl<'a> Encode for Cow<'a, str> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(self)
    }

    fn to_encode_buf(&self) -> Box<EncodeBuf> {
        unimplemented!()
        // box self.to_owned()
    }
}

impl Encode for String {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(&self[..])
    }

    fn to_encode_buf(&self) -> Box<EncodeBuf> {
        box self.to_owned()
    }
}

impl<W: Write> Encoder for W {
    fn encode_bool(&mut self, value: bool) -> Result<(), Error> {
        write!(self, "{}", value)
    }

    fn encode_str(&mut self, value: &str) -> Result<(), Error> {
        write!(self, "{}", value)
    }
}

// for token in tokens:
// match token {
//  Lit(l) => wr.write_all(l);
//  Message(msg) => wr.write_all(msg);
//  Meta(name, value) => {
//    wr.write_all(name);
//    wr.write_all(": ");
//    value.encode(&mut wr);
//  }
//}

#[derive(Debug)]
pub struct Record<'a> {
    timestamp: DateTime<UTC>,
    severity: Severity,
    format: Arguments<'a>,
    // TODO: thread: usize,
    // TODO: module: &'static str,
    // TODO: line: u32,
    meta: &'a MetaList<'a>,
}

#[derive(Debug)]
pub struct RecordBuf {
    timestamp: DateTime<UTC>,
    severity: Severity,
    message: String,
    /// Ordered from recently added.
    meta: Vec<MetaBuf>,
}

impl RecordBuf {
    pub fn new(severity: Severity, message: String, meta: Vec<MetaBuf>) -> RecordBuf {
        RecordBuf {
            timestamp: UTC::now(),
            severity: severity,
            message: message,
            meta: meta,
        }
    }
}

enum Event {
    Record(RecordBuf),
    // Reset(Vec<Handler>),
    // Filter(Filter),
    Shutdown,
}

struct Inner {
    severity: AtomicIsize,
    tx: Mutex<mpsc::Sender<Event>>,
    thread: Option<JoinHandle<()>>,
}

impl Inner {
    fn new(tx: mpsc::Sender<Event>, rx: mpsc::Receiver<Event>) -> Inner {
        let thread = thread::spawn(move || {
            for event in rx {
                match event {
                    Event::Record(rec) => {
                        // println!("{:?}", rec);
                    }
                    Event::Shutdown => break,
                }
            }
        });

        Inner {
            severity: AtomicIsize::new(0),
            tx: Mutex::new(tx),
            thread: Some(thread),
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.tx.lock().unwrap().send(Event::Shutdown).unwrap();
        self.thread.take().unwrap().join().unwrap();
    }
}

#[derive(Clone)] // TODO: Test.
pub struct Logger {
    tx: mpsc::Sender<Event>,
    inner: Arc<Inner>,
}

impl Logger {
    pub fn new() -> Logger {
        let (tx, rx) = mpsc::channel();

        Logger {
            tx: tx.clone(),
            inner: Arc::new(Inner::new(tx, rx)),
        }
    }

    fn log<'a>(&self, sev: Severity, format: Arguments<'a>, meta: &MetaList<'a>) {
        if sev >= self.inner.severity.load(Ordering::Relaxed) {
            let record = RecordBuf::new(sev, format!("{}", format), From::from(meta));

            if let Err(..) = self.tx.send(Event::Record(record)) {
                // TODO: Return error.
            }
        }
    }
}

// #[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {
        $log.log($sev, format_args!($fmt, $($args)*), &MetaList::new(
            &[$(Meta::new(stringify!($name), &$val)),*]
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
    use super::{Logger, Meta, MetaList, Encode};

    #[cfg(feature="benchmark")]
    use test::Bencher;

    #[test]
    fn logger_send() {
        fn checker<T: Send>(v: T) {}

        let log = Logger::new();
        checker(log.clone());
    }

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
            path: "/home/esafronov", // Duplicates are allowed as a stacking feature.
            target: "core",
            owned: "le message".to_string(),
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message(b: &mut Bencher) {
        let log = Logger::new();

        b.iter(|| {
            log!(log, 0, "file does not exist: /var/www/favicon.ico");
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message_with_meta1(b: &mut Bencher) {
        let log = Logger::new();

        b.iter(|| {
            log!(log, 0, "file does not exist: /var/www/favicon.ico", {
                path: "/home",
            });
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message_with_meta6(b: &mut Bencher) {
        let log = Logger::new();

        b.iter(|| {
            log!(log, 0, "file does not exist: /var/www/favicon.ico", {
                path1: "/home1",
                path2: "/home2",
                path3: "/home3",
                path4: "/home4",
                path5: "/home5",
                path6: "/home6",
            });
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message_with_format_and_meta6(b: &mut Bencher) {
        let log = Logger::new();

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
