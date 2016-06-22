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

// TODO: impl Iterator<Item=Meta> for RecordIter<'a> {}

#[derive(Debug, Copy, Clone)]
pub struct Context {
    thread: usize,
    module: &'static str,
    line: u32,
}

// TODO: When filtering we can pass both Record and RecordBuf. That's why we need a trait to union
// them.
#[derive(Debug)]
pub struct Record<'a> {
    timestamp: DateTime<UTC>, // TODO: Consumes about 25ns. Also it may be useful to obtain local time.
    severity: Severity,
    context: Context,
    format: Arguments<'a>,
    meta: &'a MetaList<'a>,
}

#[derive(Debug)]
pub struct RecordBuf {
    timestamp: DateTime<UTC>,
    severity: Severity,
    context: Context,
    message: String,
    /// Ordered from recently added.
    meta: Vec<MetaBuf>,
}

impl<'a> From<&'a Record<'a>> for RecordBuf {
    fn from(val: &'a Record<'a>) -> RecordBuf {
        RecordBuf {
            timestamp: val.timestamp,
            severity: val.severity,
            context: val.context,
            message: format!("{}", val.format),
            meta: From::from(val.meta),
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
                        println!("{:?}", rec);
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

pub trait Logger {
    fn log<'a>(&self, record: &Record<'a>);
}

#[derive(Clone)]
pub struct AsyncLogger {
    tx: mpsc::Sender<Event>,
    inner: Arc<Inner>,
}

struct Scope<'a, F: FnOnce() -> &'static str> {
    logger: &'a Logger,
    f: F,
}

#[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {{
        extern crate chrono;

        use chrono::UTC;
        use $crate::{Context, Logger, Record};

        let context = Context {
            thread: 0,
            module: module_path!(),
            line: line!(),
        };

        $log.log(&Record {
            timestamp: UTC::now(),
            severity: $sev,
            context: context,
            format: format_args!($fmt, $($args)*),
            meta: &$crate::MetaList::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ]),
        });
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

impl<'a, F: FnOnce() -> &'static str> Drop for Scope<'a, F> {
    fn drop(&mut self) {
        let l = &self.logger;
        log!(l, 42, "fuck you");
    }
}

impl AsyncLogger {
    pub fn new() -> AsyncLogger {
        let (tx, rx) = mpsc::channel();

        AsyncLogger {
            tx: tx.clone(),
            inner: Arc::new(Inner::new(tx, rx)),
        }
    }

    fn scoped<F: FnOnce() -> &'static str>(&self, f: F) -> Scope<F> {
        Scope {
            logger: self as &Logger,
            f: f,
        }
    }
}

impl Logger for AsyncLogger {
    fn log<'a>(&self, record: &Record<'a>) {
        if record.severity >= self.inner.severity.load(Ordering::Relaxed) {
            if let Err(..) = self.tx.send(Event::Record(RecordBuf::from(record))) {
                // TODO: Return error.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncLogger, Meta, MetaList, Encode};

    #[cfg(feature="benchmark")]
    use test::Bencher;

    #[test]
    fn logger_send() {
        fn checker<T: Send>(_v: T) {}

        let log = AsyncLogger::new();
        checker(log.clone());
    }

    #[test]
    fn log() {
        let log = AsyncLogger::new();

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

        log.scoped(move || "wow");
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message(b: &mut Bencher) {
        let log = AsyncLogger::new();

        b.iter(|| {
            log!(log, 0, "file does not exist: /var/www/favicon.ico");
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message_with_meta1(b: &mut Bencher) {
        let log = AsyncLogger::new();

        b.iter(|| {
            log!(log, 0, "file does not exist: /var/www/favicon.ico", {
                path: "/home",
            });
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_log_message_with_meta6(b: &mut Bencher) {
        let log = AsyncLogger::new();

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
        let log = AsyncLogger::new();

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
