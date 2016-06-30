use super::Record;

mod terminal;

/// Outputs are responsible for delivering log events to their destination.
pub trait Output {
    fn write(&mut self, record: &Record, message: &[u8]) -> Result<(), ::std::io::Error>;
}
