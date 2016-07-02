use super::Record;

mod terminal;

pub use self::terminal::Terminal;

/// Outputs are responsible for delivering formatted log events to their destination.
pub trait Output: Send + Sync {
    fn write(&self, rec: &Record, message: &[u8]) -> Result<(), ::std::io::Error>;
}
