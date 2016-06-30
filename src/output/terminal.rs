use std::io::Write;

use Record;

use {Output};

struct Terminal;

impl Output for Terminal {
    fn write(&mut self, _record: &Record, message: &[u8]) -> Result<(), ::std::io::Error> {
        ::std::io::stdout().write_all(message)
    }
}
