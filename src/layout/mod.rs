use std::io::Write;

use super::Record;

mod pattern;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: ::std::io::Error) {
            from()
        }
        AttributeNotFound {}
    }
}

pub trait Layout {
    fn format(&self, rec: &Record, wr: &mut Write) -> Result<(), Error>;
}

// TODO: Temporary.
pub use self::pattern::PatternLayout;
