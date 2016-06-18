#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(plugin)]
#![feature(question_mark)]

#![plugin(peg_syntax_ext)]

extern crate chrono;
#[macro_use] extern crate quick_error;
#[cfg(feature="benchmark")] extern crate test;

pub mod appender;
pub mod layout;
pub mod logger;
mod record;
mod severity;

pub use self::record::Record;
pub use self::severity::Severity;
pub use self::logger::Logger;
