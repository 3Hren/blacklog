use std::fmt::Arguments;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicIsize, Ordering};

use filter::{Filter, FilterAction, NullFilter};
use logger::Logger;
use record::Record;

/// Extends the given logger with an ability to filter incoming events.
///
/// # Note
///
/// The logger filter acts like a function to make filtering things common, but this may be
/// significant performance overhead for denied events, because to obtain a filter we mush lock a
/// mutex and copy a shared pointer containing the filter.
#[derive(Clone)]
pub struct FilteredLoggerAdapter<L> {
    logger: L,
    filter: Arc<Mutex<Arc<Box<Filter>>>>,
}

impl<L: Logger> FilteredLoggerAdapter<L> {
    /// Constructs an adaptor by wrapping the given logger.
    ///
    /// By default a NullFilter is set, which is neutral to all records passed.
    pub fn new(logger: L) -> FilteredLoggerAdapter<L> {
        FilteredLoggerAdapter {
            logger: logger,
            filter: Arc::new(Mutex::new(Arc::new(box NullFilter))),
        }
    }

    /// Replaces the current filter with the given one.
    pub fn filter(&self, filter: Box<Filter>) {
        *self.filter.lock().unwrap() = Arc::new(filter);
    }
}

impl<L: Logger> Logger for FilteredLoggerAdapter<L> {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        let filter = self.filter.lock().unwrap().clone();

        match filter.filter(&rec) {
            FilterAction::Deny => {}
            FilterAction::Accept | FilterAction::Neutral => {
                self.logger.log(rec, args)
            }
        }
    }
}

/// Extends the given logger with an ability to fast filter incoming events by their severity value.
///
/// Acts like a `FilteredLoggerAdapter`, but much more faster.
#[derive(Clone)]
pub struct SeverityFilteredLoggerAdapter<L> {
    logger: L,
    threshold: Arc<AtomicIsize>,
}

impl<L: Logger> SeverityFilteredLoggerAdapter<L> {
    /// Constructs an adaptor by wrapping the given logger.
    ///
    /// By default a 0 value is set as a threshold.
    pub fn new(logger: L) -> SeverityFilteredLoggerAdapter<L> {
        SeverityFilteredLoggerAdapter {
            logger: logger,
            threshold: Arc::new(AtomicIsize::new(0)),
        }
    }

    /// Replaces the current threshold with the given one.
    pub fn filter(&self, value: i32) {
        self.threshold.store(value as isize, Ordering::Release);
    }
}

impl<L: Logger> Logger for SeverityFilteredLoggerAdapter<L> {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        if rec.severity() >= self.threshold.load(Ordering::Relaxed) as i32 {
            self.logger.log(rec, args)
        }
    }
}
