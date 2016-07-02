use super::Record;

mod term;

pub use self::term::Term;

/// Outputs are responsible for delivering formatted log events to their destination.
pub trait Output: Send + Sync {
    fn write(&self, rec: &Record, message: &[u8]) -> Result<(), ::std::io::Error>;
}
