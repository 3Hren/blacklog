use std::io::Write;

use super::Record;

mod pattern;

pub trait Layout {
    fn format(&mut self, rec: &Record, wr: &mut Write);
}

// TODO: Temporary.
pub use self::pattern::PatternLayout;
