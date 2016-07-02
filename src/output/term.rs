use std::error;
use std::io::Write;

use {Config, Output, Record, Registry};

use factory::Factory;

pub struct Term;

impl Output for Term {
    fn write(&self, _rec: &Record, message: &[u8]) -> Result<(), ::std::io::Error> {
        let stdout = ::std::io::stdout();
        let mut wr = stdout.lock();
        wr.write_all(message)?;
        wr.write_all(b"\n")
    }
}

impl Factory for Term {
    type Item = Output;

    fn ty() -> &'static str {
        "term"
    }

    fn from(_cfg: &Config, _registry: &Registry) -> Result<Box<Output>, Box<error::Error>> {
        Ok(box Term)
    }
}
