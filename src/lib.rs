#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(integer_atomics)]
#![feature(plugin)]
#![feature(question_mark)]
#![feature(unicode)]

#![plugin(peg_syntax_ext)]

#[cfg(unix)] extern crate libc;
extern crate chrono;
extern crate serde_json;
#[macro_use] extern crate quick_error;
#[cfg(feature="benchmark")] extern crate test;

mod layout;
mod meta;
mod output;
mod record;
mod registry;
mod severity;
mod thread;

pub use self::layout::{Layout, LayoutFactory};
pub use self::meta::{FnMeta, Meta, MetaBuf, MetaList};
pub use self::meta::format::{Format, Formatter, IntoBoxedFormat};
pub use self::output::Output;
pub use self::record::{Record, InactiveRecord};
pub use self::registry::{Config, Registry};
pub use self::severity::Severity;

mod _wip;
