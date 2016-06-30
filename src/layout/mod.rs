use std::io::Write;

use registry::Config;

use super::Record;

mod pattern;

pub use self::pattern::PatternLayoutFactory;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: ::std::io::Error) {
            from()
        }
        MetaNotFound {} // TODO: What meta?
    }
}

/// Layouts are responsible for formatting a log event into a form that meets the needs of whatever
/// will be consuming the log event.
pub trait Layout: Send + Sync {
    fn format(&self, rec: &Record, wr: &mut Write) -> Result<(), Error>;
}

pub trait LayoutFactory {
    /// Returns type as a string that is used mainly for concrete layout identification.
    fn ty() -> &'static str where Self: Sized;

    /// Constructs a new layout by configuring it with the given config.
    fn from(&self, cfg: &Config) -> Result<Box<Layout>, Box<::std::error::Error>>;
}
