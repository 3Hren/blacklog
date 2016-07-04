use record::Record;

mod null;

pub use self::null::NullFilter;

/// Filtering result.
pub enum FilterAction {
    /// The record should be dropped immediately.
    Deny,
    /// The record should be accepted immediately.
    Accept,
    /// Filter doesn't know whether to drop or to accept a record.
    ///
    /// It is possible to combine multiple filters into the single one, which requires in its turn
    /// a neutral value to be able to advance filtering if the current filter returns neutral value.
    Neutral,
}

/// Filters are responsible for filtering logging events.
///
/// Filters may be configured in one of three locations:
///
/// 1. Logger filters are configured on a specified Logger. Events that are rejected by these
///    filters will be discarded.
/// 2. Handle filters are used to determine if a specific Handle should handle the formatting and
///    publication of the event.
/// 3. Output filters are used to determine if a logger should route the event to an output.
pub trait Filter: Send + Sync {
    fn filter(&self, rec: &Record) -> FilterAction;
}

impl<F> Filter for F
    where F: Fn(&Record) -> FilterAction + Send + Sync
{
    fn filter(&self, rec: &Record) -> FilterAction {
        self(rec)
    }
}
