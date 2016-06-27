use std::fmt::Arguments;
use std::borrow::Cow;
use std::iter::Iterator;

use chrono::{DateTime, UTC};

use Meta;

use super::meta::{MetaBuf, MetaList};

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

// TODO: When filtering we can pass both Record and RecordBuf. That's why we may need a trait to
// unite them.
#[derive(Debug, Copy, Clone)]
pub struct InactiveRecord<'a> {
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

    pub fn timestamp(&self) -> &DateTime<UTC> {
        &self.timestamp
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
                id if id == metalist.meta().len() - 1 => {
                    let res = metalist.meta()[id];
                    self.id = 0;
                    self.curr = metalist.prev();
                    Some(res)
                }
                id if id < metalist.meta().len() - 1 => {
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

#[cfg(test)]
mod tests {
    use super::super::meta::{Meta, MetaList};
    use super::{Record};

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
}
