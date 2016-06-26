#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(integer_atomics)]
#![feature(plugin)]
#![feature(question_mark)]

#![plugin(peg_syntax_ext)]

#[cfg(unix)]
extern crate libc;
extern crate chrono;
extern crate serde_json;
#[macro_use] extern crate quick_error;
#[cfg(feature="benchmark")] extern crate test;

// pub mod appender;
mod layout;
mod meta;
mod record;
mod registry;
mod severity;
mod thread;

pub use self::layout::{Layout, LayoutFactory};
pub use self::meta::{Logger, Meta, MetaList};
pub use self::meta::Encode;
pub use self::record::{Record, InactiveRecord};
pub use self::registry::Registry;
pub use self::severity::Severity;
