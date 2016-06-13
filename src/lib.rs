#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(plugin)]
#![feature(question_mark)]

#![plugin(peg_syntax_ext)]

#[cfg(feature="benchmark")] extern crate test;
#[macro_use] extern crate quick_error;

pub mod layout;
mod record;
mod severity;

pub use self::record::Record;
pub use self::severity::Severity;
