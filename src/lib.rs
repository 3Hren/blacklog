#![cfg_attr(feature="benchmark", feature(test))]

#![feature(box_syntax)]
#![feature(plugin)]
#![feature(question_mark)]
#![feature(unicode)]

#![plugin(peg_syntax_ext)]

#[cfg(unix)] extern crate libc;
#[cfg(feature="benchmark")] extern crate test;
extern crate chrono;
extern crate serde_json;
#[macro_use] extern crate quick_error;
extern crate log;

mod factory;
pub mod filter;
pub mod handle;
pub mod layout;
pub mod logger;
mod meta;
pub mod output;
mod record;
mod registry;
mod severity;
mod thread;

pub use self::filter::Filter;
pub use self::handle::Handle;
pub use self::layout::Layout;
pub use self::logger::Logger;
pub use self::meta::{FnMeta, Meta, MetaBuf, MetaLink};
pub use self::meta::format::{Format, Formatter, IntoBoxedFormat};
pub use self::output::Output;
pub use self::record::{Record};
pub use self::registry::{Config, Registry};
pub use self::severity::Severity;

// mod _wip;
