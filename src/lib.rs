#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(integer_atomics)]
#![feature(plugin)]
#![feature(question_mark)]

#![plugin(peg_syntax_ext)]

#[cfg(unix)]
extern crate libc;
extern crate chrono;
#[macro_use] extern crate quick_error;
#[cfg(feature="benchmark")] extern crate test;

// pub mod appender;
pub mod layout;
mod severity;
mod meta;
mod thread;
mod record;

pub use self::severity::Severity;
pub use self::meta::Encode;
pub use self::meta::{Logger, Meta, MetaList};
pub use self::record::{Record, InactiveRecord};
