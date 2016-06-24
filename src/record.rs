use std::fmt::Arguments;
use std::borrow::Cow;

use chrono::{DateTime, UTC};

use super::meta::{Meta, MetaBuf, MetaList};

// TODO: impl Iterator<Item=Meta> for RecordIter<'a> {}

/// Logging event context contains an information about where the event was created including the
/// source code location and thread number.
#[derive(Debug, Copy, Clone)]
struct Context {
    /// The line number on which the logging event was created.
    line: u32,
    /// The module path where the logging event was created.
    module: &'static str,
    /// The thread id where the logging event was created.
    thread: usize,
}

// TODO: When filtering we can pass both Record and RecordBuf. That's why we need a trait to unite
// them.
#[derive(Debug, Copy, Clone)]
pub struct FrozenRecord<'a> {
    sev: i32,
    context: Context,
    format: Arguments<'a>, // TODO: enum Message { Ready(&'a str), Prepared(Arguments<'a>) }.
    meta: &'a MetaList<'a>,
}

#[derive(Debug, Copy, Clone)]
enum Message<'a> {
    Ready(&'a str),
    Readonly(&'static str),
}

#[derive(Debug, Clone)]
pub struct Record<'a> {
    sev: i32,
    message: Cow<'static, str>,
    timestamp: DateTime<UTC>,
    context: Context,
    meta: &'a MetaList<'a>,
}

impl<'a> Record<'a> {
    pub fn new<T>(sev: T, line: u32, module: &'static str, format: Arguments<'a>, meta: &'a MetaList<'a>) -> FrozenRecord<'a>
        where i32: From<T>
    {
        let context = Context {
            line: line,
            module: module,
            thread: super::thread::id(),
        };

        FrozenRecord {
            sev: From::from(sev),
            context: context,
            format: format,
            meta: meta,
        }
    }

    pub fn severity(&self) -> i32 {
        self.sev
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl<'a> FrozenRecord<'a> {
    #[inline]
    pub fn activate(self) -> Record<'a> {

        Record {
            sev: self.sev,
            message: Cow::Owned(format!("{}", self.format)),
            timestamp: UTC::now(),
            context: self.context,
            meta: self.meta,
        }
    }

    pub fn severity(&self) -> i32 {
        self.sev
    }
}

#[derive(Debug)]
pub struct RecordBuf {
    timestamp: DateTime<UTC>,
    sev: i32,
    context: Context,
    message: Cow<'static, str>,
    /// Ordered from recently added.
    meta: Vec<MetaBuf>,
}

impl<'a> From<Record<'a>> for RecordBuf {
    fn from(val: Record<'a>) -> RecordBuf {
        RecordBuf {
            timestamp: val.timestamp,
            sev: val.sev,
            context: val.context,
            message: val.message,
            meta: From::from(val.meta),
        }
    }
}
