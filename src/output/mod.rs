use super::Record;

mod terminal;

pub use self::terminal::TerminalOutputFactory;

/// Outputs are responsible for delivering formatted log events to their destination.
pub trait Output {
    fn write(&self, record: &Record, message: &[u8]) -> Result<(), ::std::io::Error>;
}
