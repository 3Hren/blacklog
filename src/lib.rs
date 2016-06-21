#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(plugin)]
#![feature(question_mark)]

#![plugin(peg_syntax_ext)]

extern crate chrono;
#[macro_use] extern crate quick_error;
#[cfg(feature="benchmark")] extern crate test;

// pub mod appender;
pub mod layout;
mod logger; // TODO: pub.
mod record;
mod severity;
mod meta;

pub use self::record::Record;
pub use self::severity::Severity;
// pub use self::logger::Logger;
pub use self::meta::{Meta, MetaList, Value};
