use super::Record;

mod terminal;

// TODO: Better naming not to confuse log4j2 users? There are appender + layout, but we have
// handler = layout + [appender].
pub trait Appender {
    fn append(record: &Record, message: &[u8]) -> Result<(), ::std::io::Error>;
}
