use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::{self, JoinHandle};

use {Record, Severity};

use super::record::RecordBuf;

use {Meta};
use {Format, Formatter, IntoBoxedFormat};

use meta::FnMeta;
use meta::format::FormatInto;

type Error = ::std::io::Error;

trait Mutant : Send + Sync {
    fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record));
}

struct FalloutMutant;

impl FalloutMutant {
    fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record)) {
        let v = 42;
        let m = &[Meta::new("a1", &v)];
        // let meta = MetaLink::next(m, Some(rec.meta));
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

enum Event {
    Record(RecordBuf),
    // Reset(Vec<Handler>),
    // Filter(Filter),
    Shutdown,
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
