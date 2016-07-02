use Record;

mod sync;

pub use self::sync::SyncHandle;

/// Combines a filter, layout and outputs together.
///
/// Handles are responsible for combining a filter, layout and many outputs together becoming an
/// entry point for logging event after primary filtering in the logger.
pub trait Handle: Send + Sync {
    /// Handles the given record.
    ///
    /// Typically this method should filter, mutate and format a record and send it to one or many
    /// outputs. Implementations are free to do it either in synchronous or asynchronous way.
    ///
    /// Note, that filtering out a record is not considered as error.
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error>;
}
