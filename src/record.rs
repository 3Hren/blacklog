use std::fmt::Arguments;
use std::borrow::Cow;
use std::iter::Iterator;

use chrono::{DateTime, UTC};

use {Meta, MetaBuf, MetaList};

/// Logging event context contains an information about where the event was created including the
/// source code location and thread number.
#[derive(Copy, Clone)]
struct Context {
    /// The line number on which the logging event was created.
    line: u32,
    /// The module path where the logging event was created.
    module: &'static str,
    /// The thread id where the logging event was created.
    thread: usize,
}

// TODO: When filtering we can pass both Record and RecordBuf. That's why we may need a trait to
// unite them.
#[derive(Copy, Clone)]
pub struct InactiveRecord<'a> {
    sev: i32,
    format: Arguments<'a>,
    context: Context,
    meta: &'a MetaList<'a>,
}

// TODO: Zero-copy optimization.
// #[derive(Copy, Clone)]
// enum Message<'a> {
//     Ready(&'a str),
//     Readonly(&'static str),
// }

#[derive(Clone)]
pub struct Record<'a> {
    sev: i32,
    message: Cow<'static, str>,
    timestamp: DateTime<UTC>,
    context: Context,
    meta: &'a MetaList<'a>,
}

impl<'a> Record<'a> {
    pub fn new<T>(sev: T, line: u32, module: &'static str, format: Arguments<'a>, meta: &'a MetaList<'a>) -> InactiveRecord<'a>
        where i32: From<T>
    {
        let context = Context {
            line: line,
            module: module,
            thread: super::thread::id(),
        };

        InactiveRecord {
            sev: From::from(sev),
            format: format,
            context: context,
            meta: meta,
        }
    }

    fn from_owned(rec: &'a RecordBuf, metalist: &'a MetaList<'a>) -> Record<'a> {
        Record {
            sev: rec.sev,
            message: rec.message.clone(),
            timestamp: rec.timestamp,
            context: rec.context,
            meta: metalist,
        }
    }

    pub fn severity(&self) -> i32 {
        self.sev
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn timestamp(&self) -> &DateTime<UTC> {
        &self.timestamp
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

    pub fn iter(&self) -> RecordIter<'a> {
        RecordIter::new(self.meta)
    }
}

pub struct RecordIter<'a> {
    metalist: &'a MetaList<'a>,
    id: usize,
    curr: Option<&'a MetaList<'a>>,
}

impl<'a> RecordIter<'a> {
    fn new(metalist: &'a MetaList) -> RecordIter<'a> {
        RecordIter {
            metalist: metalist,
            id: 0,
            curr: Some(metalist),
        }
    }
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Meta<'a>;

    fn next(&mut self) -> Option<Meta<'a>> {
        self.curr.and_then(|metalist| {
            match self.id {
                id if id + 1 == metalist.meta().len() => {
                    let res = metalist.meta()[id];
                    self.id = 0;
                    self.curr = metalist.prev();
                    Some(res)
                }
                id if id + 1 < metalist.meta().len() => {
                    let res = metalist.meta()[id];
                    self.id += 1;
                    Some(res)
                }
                _ => None
            }
        })
    }
}

// TODO: impl ExactSizeIterator, DoubleEndedIterator, IntoIterator, FromIterator.

impl<'a> InactiveRecord<'a> {
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

impl<'a> From<&'a Record<'a>> for RecordBuf {
    fn from(val: &'a Record<'a>) -> RecordBuf {
        RecordBuf {
            timestamp: val.timestamp,
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

    #[cfg(feature="benchmark")]
    use test::Bencher;

    #[test]
    fn inactive_severity() {
        assert_eq!(0, Record::new(0, 0, "", format_args!(""), &MetaList::new(&[])).severity());
    }

    #[test]
    fn severity() {
        assert_eq!(0, Record::new(0, 0, "", format_args!(""), &MetaList::new(&[]))
            .activate()
            .severity());
    }

    #[test]
    fn iter() {
        assert_eq!(4, Record::new(0, 0, "", format_args!(""), &MetaList::new(&[
            Meta::new("n#1", &"v#1"),
            Meta::new("n#2", &"v#2"),
            Meta::new("n#3", &"v#3"),
            Meta::new("n#4", &"v#4"),
        ])).activate().iter().count());
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

        run(&Record::new(0, 0, "", format_args!(""), &metalist2).activate());
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

        run(&Record::new(1, 2, "mod", format_args!("message"), &metalist).activate());
    }
}
