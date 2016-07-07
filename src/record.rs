use std::fmt::Arguments;
use std::borrow::Cow;

use chrono::{DateTime, UTC};

use {MetaBuf, MetaList};

use meta::MetaListIter;

/// Logging event context contains an information about where the event was created including the
/// source code location and thread id.
#[derive(Copy, Clone)]
struct Context {
    /// The line number on which the logging event was created.
    line: u32,
    /// The module path where the logging event was created.
    module: &'static str,
    /// The thread id where the logging event was created.
    thread: usize,
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
#[derive(Clone)]
pub struct Record<'a> {
    // TODO: Maybe it's reasonable to keep this i32 + &'static Format to make severity formattable
    // without explicit function provisioning in layouts.
    sev: i32,
    message: Cow<'static, str>,
    timestamp: Option<DateTime<UTC>>,
    context: Context,
    meta: &'a MetaList<'a>,
}

impl<'a> Record<'a> {
    pub fn new<T>(sev: T, line: u32, module: &'static str, meta: &'a MetaList<'a>) -> Record<'a>
        where i32: From<T>
    {
        let context = Context {
            line: line,
            module: module,
            thread: super::thread::id(),
        };

        Record {
            sev: From::from(sev),
            message: Cow::Borrowed(""),
            timestamp: None,
            context: context,
            meta: meta,
        }
    }

    fn from_owned(rec: &'a RecordBuf, metalist: &'a MetaList<'a>) -> Record<'a> {
        Record {
            sev: rec.sev,
            message: rec.message.clone(),
            timestamp: Some(rec.timestamp),
            context: rec.context,
            meta: metalist,
        }
    }

    /// Returns a severity number as `i32` that was set during this record creation.
    pub fn severity(&self) -> i32 {
        self.sev
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn timestamp(&self) -> &DateTime<UTC> {
        // TODO: Bettern to return by value then.
        &self.timestamp.as_ref().unwrap()
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
    /// As a record contains optionally chained lists of meta information (aka attributes), we can
    /// iterate through in direct order there were chained to emulate some kind of priorities. This
    /// method returns such an iterator.
    pub fn iter(&self) -> MetaListIter<'a> {
        self.meta.iter()
    }

    pub fn activate<'b>(&mut self, format: Arguments<'b>) {
        self.message = Cow::Owned(format!("{}", format));
        self.timestamp = Some(UTC::now());
    }
}

// TODO: impl ExactSizeIterator, DoubleEndedIterator, IntoIterator, FromIterator.

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
            timestamp: val.timestamp.unwrap(),
            sev: val.sev,
            context: val.context,
            message: val.message,
            meta: From::from(val.meta),
        }
    }
}

impl<'a> From<&'a Record<'a>> for RecordBuf {
    fn from(val: &'a Record<'a>) -> RecordBuf {
        RecordBuf {
            timestamp: val.timestamp.unwrap(),
            sev: val.sev,
            context: val.context,
            message: val.message.clone(),
            meta: From::from(val.meta),
        }
    }
}

#[cfg(test)]
mod tests {
    use {Meta, MetaList};
    use super::*;

    // #[cfg(feature="benchmark")]
    // use test::Bencher;

    #[test]
    fn severity() {
        assert_eq!(0, Record::new(0, 0, "", &MetaList::new(&[])).severity());
    }

    #[test]
    fn iter() {
        assert_eq!(4, Record::new(0, 0, "", &MetaList::new(&[
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

            assert_eq!("n#3", iter.next().unwrap().name);
            assert_eq!("n#4", iter.next().unwrap().name);
            assert_eq!("n#1", iter.next().unwrap().name);
            assert_eq!("n#2", iter.next().unwrap().name);
            assert!(iter.next().is_none());
        }

        let v = 42;
        let meta1 = &[Meta::new("n#1", &v), Meta::new("n#2", &v)];
        let meta2 = &[Meta::new("n#3", &v), Meta::new("n#4", &v)];
        let metalist1 = MetaList::new(meta1);
        let metalist2 = MetaList::next(meta2, Some(&metalist1));

        run(&Record::new(0, 0, "", &metalist2));
    }

    #[test]
    fn to_owned() {
        fn run(rec: &Record) {
            let owned = RecordBuf::from(rec);
            let meta = owned.meta.iter().map(Into::into).collect::<Vec<Meta>>();
            let metalist = MetaList::new(&meta);
            let borrow = Record::from_owned(&owned, &metalist);

            assert_eq!(1, borrow.severity());
            assert_eq!("message", borrow.message());
            assert_eq!(rec.timestamp(), borrow.timestamp());
            assert_eq!(2, borrow.line());
            assert_eq!("mod", borrow.module());
            assert_eq!(rec.thread(), borrow.thread());

            let mut iter = borrow.iter();
            assert_eq!("n#1", iter.next().unwrap().name);
            assert_eq!("n#2", iter.next().unwrap().name);
        }

        let v = 42;
        let meta = &[Meta::new("n#1", &v), Meta::new("n#2", &v)];
        let metalist = MetaList::new(meta);

        let mut rec = Record::new(1, 2, "mod", &metalist);
        rec.activate(format_args!("message"));
        run(&rec);
    }
}
