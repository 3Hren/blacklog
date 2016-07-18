use std::io::Write;

use record::Record;

pub mod pattern;

pub use self::pattern::PatternLayout;

pub type Error = ::std::io::Error;

/// Layouts are responsible for formatting a log event into a form that meets the needs of whatever
/// will be consuming the log event.
pub trait Layout: Send + Sync {
    fn format(&self, rec: &Record, wr: &mut Write) -> Result<(), Error>;
}
