use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::{self, JoinHandle};

use {Record, InactiveRecord, Severity};

use super::record::RecordBuf;

use {Meta};
use {Format, Formatter, IntoBoxedFormat};

use meta::FnMeta;
use meta::format::FormatInto;

type Error = ::std::io::Error;

enum FilterAction {
    Deny,
    Accept,
    Neutral,
}

/// Filters are responsible for filtering logging events.
///
/// # Note
///
/// All filters must satisfy Sync trait to be safely usable from multiple threads.
trait Filter : Send + Sync {
    fn filter<'a>(&self, rec: &Record<'a>) -> FilterAction;
}

struct NullFilter;

impl Filter for NullFilter {
    fn filter<'a>(&self, _rec: &Record<'a>) -> FilterAction {
        FilterAction::Neutral
    }
}

pub trait Logger : Send {
    fn log<'a>(&self, record: &InactiveRecord<'a>);
}

trait Handler : Send + Sync {
    fn handle(&self, rec: &mut Record);
}

#[derive(Clone)]
struct SyncLogger {
    handlers: Arc<Vec<Box<Handler>>>,
    severity: Arc<AtomicI32>,
    filter: Arc<Mutex<Arc<Box<Filter>>>>,
}

impl SyncLogger {
    fn new(handlers: Vec<Box<Handler>>) -> SyncLogger {
        SyncLogger {
            handlers: Arc::new(handlers),
            severity: Arc::new(AtomicI32::new(0)),
            filter: Arc::new(Mutex::new(Arc::new(box NullFilter))),
        }
    }

    fn severity(&self, val: Severity) {
        self.severity.store(val, Ordering::Release);
    }

    fn filter<F>(&self, filter: F)
        where F: Filter + 'static
    {
        (*self.filter.lock().unwrap()) = Arc::new(box filter);
    }
}

trait Mutant : Send + Sync {
    fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record));
}

struct FalloutMutant;

impl FalloutMutant {
    fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record)) {
        let v = 42;
        let m = &[Meta::new("a1", &v)];
        // let meta = MetaList::next(m, Some(rec.meta));
        // let mut rec2 = *rec;
        // rec2.meta = &meta;

        // f(&mut rec2)
        f(rec)
    }
}

trait Layout {}

struct SomeHandler {
    // layout: Box<Layout>,
    mutants: Arc<Vec<Box<Mutant>>>,
    // appenders: Vec<Box<Appender>>,
}

impl SomeHandler {
    fn handle_<'a>(&self, rec: &mut Record<'a>, mutants: &[Box<Mutant>]) {
        println!("{:?}", mutants.len());

        match mutants.iter().next() {
            Some(mutant) => {
                mutant.mutate(rec, &|rec| {
                    self.handle_(rec, &mutants[1..])
                })
            }
            None => {
                let mut wr: Vec<u8> = Vec::new();
                // self.layout.format(rec, &mut wr);
            }
        }
    }
}

impl Handler for SomeHandler {
    fn handle<'a>(&self, rec: &mut Record<'a>) {
        self.handle_(rec, &self.mutants[..])
    }
}

impl Logger for SyncLogger {
    fn log<'a>(&self, record: &InactiveRecord<'a>) {
        if record.severity() >= self.severity.load(Ordering::Relaxed) {
            let record = record.activate();
            let filter = (*self.filter.lock().unwrap()).clone();

            match filter.filter(&record) {
                FilterAction::Deny => {}
                FilterAction::Accept | FilterAction::Neutral => {
                    for handler in self.handlers.iter() {
                        // copy record, make mut.
                        // add new meta.
                        // mutate(possible reset meta?)
                    }
                    // record.activate().
                    // .
                }
            }
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
    severity: AtomicI32,
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
            severity: AtomicI32::new(0),
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

#[derive(Clone)]
pub struct AsyncLogger {
    tx: mpsc::Sender<Event>,
    inner: Arc<Inner>,
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
    fn log<'a>(&self, record: &InactiveRecord<'a>) {
        if record.severity() >= self.inner.severity.load(Ordering::Relaxed) {
            if let Err(..) = self.tx.send(Event::Record(RecordBuf::from(record.activate()))) {
                // TODO: Return error.
            }
        }
    }
}

struct Scope<'a, F: FnOnce() -> &'static str> {
    logger: &'a Logger,
    f: F,
}

impl<'a, F: FnOnce() -> &'static str> Drop for Scope<'a, F> {
    fn drop(&mut self) {
        let l = &self.logger;
        // log!(l, 42, "fuck you");
    }
}

#[cfg(test)]
mod tests {
    #[macro_use] use logger;

    use FnMeta;
    use super::{SyncLogger, AsyncLogger, Logger};

    #[cfg(feature="benchmark")]
    use test::Bencher;

    #[test]
    fn log_with_custom_enum() {
        enum Severity {
            Debug,
            Info,
            Warn,
            Error,
        }

        impl From<Severity> for i32 {
            fn from(val: Severity) -> i32 {
                match val {
                    Severity::Debug => 0,
                    Severity::Info  => 1,
                    Severity::Warn  => 2,
                    Severity::Error => 3,
                }
            }
        }
        let log = SyncLogger::new(vec![]);

        log!(log, Severity::Debug, "file does not exist: /var/www/favicon.ico");
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
