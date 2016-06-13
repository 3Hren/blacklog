#![cfg_attr(feature="benchmark", feature(test))]

#![feature(plugin)]
#![feature(box_syntax)]
#![feature(question_mark)]

#![plugin(peg_syntax_ext)]

#[cfg(feature="benchmark")] extern crate test;

pub mod layout;
mod record;

pub use self::record::Record;
