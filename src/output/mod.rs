use super::Record;

mod file;
mod null;
mod term;

pub use self::file::FileOutput;
pub use self::term::Term;

/// Outputs are responsible for delivering formatted log events to their destination.
pub trait Output: Send + Sync {
    fn write(&self, rec: &Record, message: &[u8]) -> Result<(), ::std::io::Error>;
}
