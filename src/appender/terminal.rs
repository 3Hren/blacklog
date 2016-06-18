use std::io::Write;

use super::{Appender};
use Record;

struct Terminal;

impl Appender for Terminal {
    fn append(_record: &Record, message: &[u8]) -> Result<(), ::std::io::Error> {
        ::std::io::stdout().write_all(message)
    }
}
