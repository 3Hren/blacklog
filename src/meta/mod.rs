pub mod format;

use self::format::{FormatInto};

pub use self::format::Error;

/// Meta information (also known as attribute).
///
/// This struct represent one of the core blacklog feature - meta informations that are optionally
/// attached to every logging message and travels with it.
/// There are some predefined attributes, like message, severity, timestamp, module etc. All other
/// are represented using this struct.
#[derive(Copy, Clone)]
pub struct Meta<'a> {
    /// Name.
    pub name: &'static str,
    /// Value reference.
    pub value: &'a FormatInto,
}

impl<'a> Meta<'a> {
    #[inline]
    pub fn new(name: &'static str, value: &'a FormatInto) -> Meta<'a> {
        Meta {
            name: name,
            value: value,
        }
    }
}

/// Linked list of meta information containers. Used to composite various meta containers.
pub struct MetaList<'a> {
    prev: Option<&'a MetaList<'a>>,
    meta: &'a [Meta<'a>],
}

impl<'a> MetaList<'a> {
    #[inline]
    pub fn new(meta: &'a [Meta<'a>]) -> MetaList<'a> {
        MetaList::next(meta, None)
    }

    pub fn next(meta: &'a [Meta<'a>], prev: Option<&'a MetaList<'a>>) -> MetaList<'a> {
        MetaList {
            prev: prev,
            meta: meta,
        }
    }

    pub fn prev(&self) -> Option<&'a MetaList<'a>> {
        self.prev
    }

    pub fn meta(&self) -> &[Meta<'a>] {
        self.meta
    }

    pub fn iter(&'a self) -> MetaListIter<'a> {
        MetaListIter::new(self)
    }
}

pub struct MetaListIter<'a> {
    idx: usize,
    cur: Option<&'a MetaList<'a>>,
}

impl<'a> MetaListIter<'a> {
    fn new(metalist: &'a MetaList) -> MetaListIter<'a> {
        MetaListIter {
            idx: 0,
            cur: Some(metalist),
        }
    }
}

impl<'a> Iterator for MetaListIter<'a> {
    type Item = Meta<'a>;

    fn next(&mut self) -> Option<Meta<'a>> {
        self.cur.and_then(|metalist| {
            match self.idx {
                idx if idx + 1 == metalist.meta().len() => {
                    let res = metalist.meta()[idx];
                    self.idx = 0;
                    self.cur = metalist.prev();
                    Some(res)
                }
                idx if idx + 1 < metalist.meta().len() => {
                    let res = metalist.meta()[idx];
                    self.idx += 1;
                    Some(res)
                }
                _ => None
            }
        })
    }
}

/// Owning evil twin of Meta.
pub struct MetaBuf {
    name: &'static str,
    value: Box<FormatInto>,
}

impl MetaBuf {
    fn new(name: &'static str, value: Box<FormatInto>) -> MetaBuf {
        MetaBuf {
            name: name,
            value: value,
        }
    }
}

impl<'a> Into<Meta<'a>> for &'a MetaBuf {
    fn into(self) -> Meta<'a> {
        Meta {
            name: self.name,
            value: &*self.value,
        }
    }
}

impl<'a> From<&'a MetaList<'a>> for Vec<MetaBuf> {
    fn from(val: &'a MetaList<'a>) -> Vec<MetaBuf> {
        let mut result = Vec::with_capacity(32);

        let mut node = val;
        loop {
            for meta in node.meta.iter() {
                result.push(MetaBuf::new(meta.name, meta.value.to_boxed_format()));
            }

            if let Some(prev) = node.prev {
                node = prev;
            } else {
                break;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_name() {
        let val = 42;
        let meta = Meta::new("n#1", &val);

        assert_eq!("n#1", meta.name);
    }

    #[test]
    fn metalist_iter() {
        let val = "val";
        let meta = &[
            Meta::new("n#1", &val),
            Meta::new("n#2", &val),
            Meta::new("n#3", &val),
            Meta::new("n#4", &val),
        ];
        let metalist = MetaList::new(meta);

        assert_eq!(4, metalist.iter().count());
    }

    #[test]
    fn metalist_iter_with_nested_lists() {
        let val = 42;
        let meta1 = &[Meta::new("n#1", &val), Meta::new("n#2", &val)];
        let meta2 = &[Meta::new("n#3", &val), Meta::new("n#4", &val)];
        let metalist1 = MetaList::new(meta1);
        let metalist2 = MetaList::next(meta2, Some(&metalist1));

        let mut iter = metalist2.iter();

        assert_eq!("n#3", iter.next().unwrap().name);
        assert_eq!("n#4", iter.next().unwrap().name);
        assert_eq!("n#1", iter.next().unwrap().name);
        assert_eq!("n#2", iter.next().unwrap().name);
        assert!(iter.next().is_none());
    }
}
