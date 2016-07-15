use std::fmt::Arguments;
use std::sync::{Arc, Mutex};

use {Config, Registry};

use factory::Factory;
use handle::Handle;
use logger::Logger;
use record::Record;

/// Blocking, but still fast, thread-safe reloadable synchronous logger.
///
/// Represents a logger, which handles incoming records by sequentially iterating through the given
/// handlers.
///
/// Such kind of logger is required to implement zero-copy meta information handling, through its
/// borrowing without prior converting into owned structures. In these cases it's strongly
/// recommended that all of handlers and outputs won't block no matter what. For example UDP output
/// perfectly fits in this recommendation, but the File output - does not, which may results in
/// threads freezing in the case hardware I/O problems.
///
/// This logger acts like a root logger - the base of other functionality like filtering, which can
/// be provided by wrapping instances of this struct.
///
/// By reloading we mean that this logger can be safely reassigned in runtime, allowing both to
/// change configuration and to correctly finish all outstanding operations, like flushing. This
/// feature gives an ability to implement popular SIGHUP logging rotation.
#[derive(Clone)]
pub struct SyncLogger {
    handlers: Arc<Mutex<Arc<Vec<Box<Handle>>>>>,
}

impl SyncLogger {
    pub fn new(handlers: Vec<Box<Handle>>) -> SyncLogger {
        SyncLogger {
            handlers: Arc::new(Mutex::new(Arc::new(handlers))),
        }
    }

    pub fn reset(&self, handlers: Vec<Box<Handle>>) {
        *self.handlers.lock().unwrap() = Arc::new(handlers);
    }
}

impl Logger for SyncLogger {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        // TODO: Maybe check whether a record was activated before.
        rec.activate(args);

        let handlers = self.handlers.lock().unwrap();
        for handle in handlers.iter() {
            handle.handle(rec).unwrap();
        }
    }
}

impl Factory for SyncLogger {
    type Item = Logger;

    fn ty() -> &'static str {
        "sync"
    }

    fn from(cfg: &Config, registry: &Registry) -> Result<Box<Logger>, Box<::std::error::Error>> {
        let handlers = cfg.find("handlers")
            .ok_or("field \"handlers\" is required")?
            .as_array()
            .ok_or("field \"handlers\" must be an array")?
            .iter()
            .map(|cfg| registry.handle(cfg))
            .collect()?;

        let res = box SyncLogger::new(handlers);

        Ok(res)
    }
}
