use std::io::Write;

use {Config, Output, Record};

use factory::Factory;

struct Terminal;

impl Output for Terminal {
    fn write(&self, _record: &Record, message: &[u8]) -> Result<(), ::std::io::Error> {
        ::std::io::stdout().write_all(message)
    }
}

pub struct TerminalOutputFactory;

impl Factory for TerminalOutputFactory {
    type Item = Output;

    fn ty() -> &'static str {
        "terminal"
    }

    fn from(&self, _cfg: &Config) -> Result<Box<Output>, Box<::std::error::Error>> {
        Ok(box Terminal)
    }
}
