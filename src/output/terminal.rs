use std::io::Write;

use {Config, Output, Record, Registry};

use factory::Factory;

struct Terminal;

impl Output for Terminal {
    fn write(&self, _rec: &Record, message: &[u8]) -> Result<(), ::std::io::Error> {
        let stdout = ::std::io::stdout();
        let mut wr = stdout.lock();
        wr.write_all(message)?;
        wr.write_all(b"\n")
    }
}

pub struct TerminalOutputFactory;

impl Factory for TerminalOutputFactory {
    type Item = Output;

    fn ty() -> &'static str {
        "terminal"
    }

    fn from(&self, _cfg: &Config, _registry: &Registry) -> Result<Box<Output>, Box<::std::error::Error>> {
        Ok(box Terminal)
    }
}
