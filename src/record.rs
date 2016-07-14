use std::fmt::Arguments;
use std::borrow::Cow;

use chrono::{DateTime, UTC};
use chrono::naive::datetime::NaiveDateTime;

use {MetaBuf, MetaLink};

use meta::{Meta, MetaLinkIter};
use meta::format::Formatter;
use severity::Severity;

/// Logging event context contains an information about where the event was created including the
/// source code location and thread id.
#[derive(Debug, Copy, Clone)]
pub struct Context {
    /// The line number on which the logging event was created.
    pub line: u32,
    /// The module path where the logging event was created.
    pub module: &'static str,
    /// The thread id where the logging event was created.
    pub thread: usize,
}

// TODO: Zero-copy optimization, but only for cases without placeholders. Don't know how to do it
// without compiler plugin for now. Or... with explicit macro syntax rules.
// #[derive(Copy, Clone)]
// enum Message<'a> {
//     Formatted(&'a str),
//     Immutable(&'static str),
// }

/// Contains all necessary information about logging event and acts like a transport.
///
/// # Note
///
/// For performance reasons all records are created in inactive state, without timestamp and
/// formatted message. It must be explicitly activated after filtering but before handling to make
/// all things act in a proper way.
pub struct Record<'a> {
    sev: i32,
    // TODO: Not sure about naming.
    sevfn: fn(i32, &mut Formatter) -> Result<(), ::std::io::Error>,
    message: Cow<'static, str>,
    timestamp: Option<DateTime<UTC>>,
    context: Context,
    metalink: &'a MetaLink<'a>, // TODO: Naming?
}

fn sevfn<T: Severity>(sev: i32, format: &mut Formatter) -> Result<(), ::std::io::Error> {
    T::format(sev, format)
}

impl<'a> Record<'a> {
    pub fn new<T>(sev: T, line: u32, module: &'static str, metalink: &'a MetaLink<'a>) -> Record<'a>
        where T: Severity + 'static
    {
        let context = Context {
            line: line,
            module: module,
            thread: super::thread::id(),
        };

        Record {
            sev: sev.num(),
            sevfn: sevfn::<T>,
            message: Cow::Borrowed(""),
            timestamp: None,
            context: context,
            metalink: metalink,
        }
    }

    /// Returns a severity number as `i32` that was set during this record creation.
    pub fn severity(&self) -> i32 {
        self.sev
    }

    // TODO: Not sure about naming. Maybe better to return severity object with .num() and format()
    //       methods.
    pub fn severity_format(&self) -> fn(i32, &mut Formatter) -> Result<(), ::std::io::Error> {
        self.sevfn
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn datetime(&self) -> DateTime<UTC> {
        self.timestamp.unwrap_or_else(|| {
            DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), UTC)
        })
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn line(&self) -> u32 {
        self.context.line
    }

    pub fn module(&self) -> &'static str {
        self.context.module
    }

    pub fn thread(&self) -> usize {
        self.context.thread
    }

    /// Returns an iterator over the meta attributes of a record.
    ///
    /// As a record contains optionally chained lists of meta information (which is also known as
    /// attributes), we can iterate through in direct order there were chained to emulate some kind
    /// of priorities. This method returns such an iterator.
    pub fn iter(&self) -> MetaLinkIter<'a> {
        self.metalink.iter()
    }

    pub fn activate<'b>(&mut self, format: Arguments<'b>) {
        // TODO: Performance!
        self.message = Cow::Owned(format!("{}", format));
        self.timestamp = Some(UTC::now());
    }
}

// TODO: impl ExactSizeIterator, DoubleEndedIterator, IntoIterator, FromIterator.

pub struct RecordBuf {
    timestamp: DateTime<UTC>,
    sev: i32,
    sevfn: fn(i32, &mut Formatter) -> Result<(), ::std::io::Error>,
    context: Context,
    message: Cow<'static, str>,
    /// Ordered from recently added.
    meta: Vec<MetaBuf>,
}

impl RecordBuf {
    pub fn borrow_and<F: Fn(&mut Record)>(&self, f: F) {
        let meta = self.meta.iter().map(Into::into).collect::<Vec<Meta>>();
        let metalink = MetaLink::new(&meta);

        let mut rec = Record {
            sev: self.sev,
            sevfn: self.sevfn,
            message: self.message.clone(),
            timestamp: Some(self.timestamp),
            context: self.context,
            metalink: &metalink,
        };

        f(&mut rec)
    }
}

impl<'a> From<&'a Record<'a>> for RecordBuf {
    fn from(val: &'a Record<'a>) -> RecordBuf {
        RecordBuf {
            timestamp: val.timestamp.unwrap(),
            sev: val.sev,
            sevfn: val.sevfn,
            context: val.context,
            message: val.message.clone(),
            meta: From::from(val.metalink),
        }
    }
}

#[cfg(test)]
mod tests {
    use {Meta, MetaLink};
    use super::*;

    #[test]
    fn severity() {
        assert_eq!(0, Record::new(0, 0, "", &MetaLink::new(&[])).severity());
    }

    #[test]
    fn iter() {
        assert_eq!(4, Record::new(0, 0, "", &MetaLink::new(&[
            Meta::new("n#1", &"v#1"),
            Meta::new("n#2", &"v#2"),
            Meta::new("n#3", &"v#3"),
            Meta::new("n#4", &"v#4"),
        ])).iter().count());
    }

    #[test]
    fn iter_with_nested_lists() {
        fn run(rec: &Record) {
            let mut iter = rec.iter();

            assert_eq!("n#1", iter.next().unwrap().name);
            assert_eq!("n#2", iter.next().unwrap().name);
            assert_eq!("n#3", iter.next().unwrap().name);
            assert_eq!("n#4", iter.next().unwrap().name);
            assert!(iter.next().is_none());
        }

        let v = 42;
        let meta1 = &[Meta::new("n#1", &v), Meta::new("n#2", &v)];
        let meta2 = &[Meta::new("n#3", &v), Meta::new("n#4", &v)];
        let metalink1 = MetaLink::new(meta1);
        let metalink2 = MetaLink::with_link(meta2, &metalink1);

        run(&Record::new(0, 0, "", &metalink2));
    }

    #[test]
    fn to_owned() {
        let v = 42;
        let meta = &[Meta::new("n#1", &v), Meta::new("n#2", &v)];
        let metalist = MetaLink::new(meta);

        let mut rec = Record::new(1, 2, "mod", &metalist);
        rec.activate(format_args!("message"));

        let owned = RecordBuf::from(&rec);

        owned.borrow_and(|borrow| {
            assert_eq!(1, borrow.severity());
            assert_eq!("message", borrow.message());
            assert_eq!(rec.datetime(), borrow.datetime());
            assert_eq!(2, borrow.line());
            assert_eq!("mod", borrow.module());
            assert_eq!(rec.thread(), borrow.thread());

            let mut iter = borrow.iter();
            assert_eq!("n#1", iter.next().unwrap().name);
            assert_eq!("n#2", iter.next().unwrap().name);
        });
    }
}
