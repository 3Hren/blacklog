use std::fmt::{self, Debug, Formatter};
use std::slice::Iter;

use self::format::FormatInto;

pub use self::format::Error;
pub use self::func::FnMeta;

pub mod format;
mod func;

/// Meta information (also known as attribute).
///
/// This struct represent one of the core blacklog feature - meta informations that are optionally
/// attached to every logging message and travels with it.
/// There are some predefined attributes: message, severity, timestamp, module, line. All other are
/// represented using this struct.
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

impl<'a> Debug for Meta<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Meta")
            .field("name", &self.name)
            .finish()
    }
}

/// Linked list of meta information containers, that are living on stack.
#[derive(Debug)]
pub struct MetaLink<'a> {
    /// Position in the linked list.
    id: usize,
    data: &'a [Meta<'a>],
    prev: Option<&'a MetaLink<'a>>,
}

impl<'a> MetaLink<'a> {
    pub fn new(data: &'a [Meta<'a>]) -> MetaLink<'a> {
        MetaLink {
            id: 0,
            data: data,
            prev: None,
        }
    }

    pub fn chained(data: &'a [Meta<'a>], prev: &'a MetaLink<'a>) -> MetaLink<'a> {
        MetaLink {
            id: 1 + prev.id,
            data: data,
            prev: Some(prev),
        }
    }

    pub fn iter(&self) -> MetaLinkIter {
        MetaLinkIter::new(self)
    }

    // TODO: pub fn rev(&self) -> RevMetaLinkIter;
}

struct LinkIter<'a> {
    id: usize,
    tail: &'a MetaLink<'a>,
}

impl<'a> LinkIter<'a> {
    fn new(tail: &'a MetaLink<'a>) -> LinkIter<'a> {
        LinkIter {
            id: 0,
            tail: tail,
        }
    }
}

impl<'a> Iterator for LinkIter<'a> {
    type Item = &'a MetaLink<'a>;

    fn next(&mut self) -> Option<&'a MetaLink<'a>> {
        if self.id > self.tail.id {
            None
        } else {
            let nadvance = self.tail.id - self.id;
            let mut id = 0;
            let mut curr = self.tail;
            for _ in 0..nadvance {
                curr = curr.prev.expect("invalid link enumeration - logic error");
            }

            self.id += 1;

            Some(curr)
        }
    }
}

pub struct MetaLinkIter<'a> {
    /// Iterator over links.
    iter: LinkIter<'a>,
    /// Iterator over meta array in the current link.
    data_iter: Iter<'a, Meta<'a>>,
}

impl<'a> MetaLinkIter<'a> {
    fn new(tail: &'a MetaLink<'a>) -> MetaLinkIter<'a> {
        let mut iter = LinkIter::new(tail);
        let curr = iter.next().expect("link must have at least one item");

        MetaLinkIter {
            iter: iter,
            data_iter: curr.data.iter(),
        }
    }
}

impl<'a> Iterator for MetaLinkIter<'a> {
    type Item = &'a Meta<'a>;

    fn next(&mut self) -> Option<&'a Meta<'a>> {
        self.data_iter.next().or_else(|| {
            self.iter.next().and_then(|link| {
                self.data_iter = link.data.iter();
                self.next()
            })
        })
    }
}

/// Linked list of meta information containers. Used to composite various meta containers.
pub struct MetaList<'a> {
    meta: &'a [Meta<'a>],
    prev: Option<&'a MetaList<'a>>,
}

impl<'a> MetaList<'a> {
    #[inline]
    pub fn new(meta: &'a [Meta<'a>]) -> MetaList<'a> {
        MetaList::next(meta, None)
    }

    pub fn next(meta: &'a [Meta<'a>], prev: Option<&'a MetaList<'a>>) -> MetaList<'a> {
        MetaList {
            meta: meta,
            prev: prev,
        }
    }

    pub fn meta(&self) -> &[Meta<'a>] {
        self.meta
    }

    pub fn prev(&self) -> Option<&'a MetaList<'a>> {
        self.prev
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
    use super::LinkIter;

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

    #[test]
    fn link_iter_single() {
        let meta = [];
        let metalink = MetaLink::new(&meta);

        assert_eq!(0, LinkIter::new(&metalink).next().unwrap().id);
        assert_eq!(1, LinkIter::new(&metalink).count());
    }

    #[test]
    fn link_iter() {
        let meta1 = [];
        let metalink1 = MetaLink::new(&meta1);
        let meta2 = [];
        let metalink2 = MetaLink::chained(&meta2, &metalink1);
        let meta3 = [];
        let metalink3 = MetaLink::chained(&meta3, &metalink2);

        let mut iter = LinkIter::new(&metalink3);

        assert_eq!(0, iter.next().unwrap().id);
        assert_eq!(1, iter.next().unwrap().id);
        assert_eq!(2, iter.next().unwrap().id);
        assert!(iter.next().is_none());

        assert_eq!(3, LinkIter::new(&metalink3).count());
    }

    #[test]
    fn metalink_iter_empty() {
        let meta = [];
        let metalink = MetaLink::new(&meta);

        assert!(metalink.iter().next().is_none());
        assert_eq!(0, metalink.iter().count());
    }

    #[test]
    fn metalink_iter_order_x() {
        let val = "";
        let meta = [
            Meta::new("n#1", &val),
            Meta::new("n#2", &val)
        ];
        let metalink = MetaLink::new(&meta);

        let mut iter = metalink.iter();

        assert_eq!("n#1", iter.next().unwrap().name);
        assert_eq!("n#2", iter.next().unwrap().name);
        assert!(iter.next().is_none());
    }

    #[test]
    fn metalink_iter_order_xy() {
        let val = "";
        let meta1 = [
            Meta::new("n#1", &val),
            Meta::new("n#2", &val),
        ];
        let metalink1 = MetaLink::new(&meta1);

        let meta2 = [
            Meta::new("n#3", &val),
            Meta::new("n#4", &val),
        ];
        let metalink2 = MetaLink::chained(&meta2, &metalink1);

        let meta3 = [
            Meta::new("n#5", &val),
            Meta::new("n#6", &val),
            Meta::new("n#7", &val),
        ];
        let metalink3 = MetaLink::chained(&meta3, &metalink2);

        let mut iter = metalink3.iter();

        assert_eq!("n#1", iter.next().unwrap().name);
        assert_eq!("n#2", iter.next().unwrap().name);
        assert_eq!("n#3", iter.next().unwrap().name);
        assert_eq!("n#4", iter.next().unwrap().name);
        assert_eq!("n#5", iter.next().unwrap().name);
        assert_eq!("n#6", iter.next().unwrap().name);
        assert_eq!("n#7", iter.next().unwrap().name);
        assert!(iter.next().is_none());
    }

    #[test]
    fn metalink_iter_order_xy_with_empty_itermediate_link() {
        let val = "";
        let meta1 = [
            Meta::new("n#1", &val),
            Meta::new("n#2", &val),
        ];
        let metalink1 = MetaLink::new(&meta1);

        let meta2 = [];
        let metalink2 = MetaLink::chained(&meta2, &metalink1);

        let meta3 = [
            Meta::new("n#5", &val),
            Meta::new("n#6", &val),
            Meta::new("n#7", &val),
        ];
        let metalink3 = MetaLink::chained(&meta3, &metalink2);

        let mut iter = metalink3.iter();

        assert_eq!("n#1", iter.next().unwrap().name);
        assert_eq!("n#2", iter.next().unwrap().name);
        assert_eq!("n#5", iter.next().unwrap().name);
        assert_eq!("n#6", iter.next().unwrap().name);
        assert_eq!("n#7", iter.next().unwrap().name);
        assert!(iter.next().is_none());
    }

    #[test]
    fn metalink_iter_order_xy_with_empty_first_link() {
        let val = "";
        let meta1 = [];
        let metalink1 = MetaLink::new(&meta1);

        let meta2 = [
            Meta::new("n#1", &val),
            Meta::new("n#2", &val),
        ];
        let metalink2 = MetaLink::chained(&meta2, &metalink1);

        let meta3 = [
            Meta::new("n#3", &val),
            Meta::new("n#4", &val),
            Meta::new("n#5", &val),
        ];
        let metalink3 = MetaLink::chained(&meta3, &metalink2);

        let mut iter = metalink3.iter();

        assert_eq!("n#1", iter.next().unwrap().name);
        assert_eq!("n#2", iter.next().unwrap().name);
        assert_eq!("n#3", iter.next().unwrap().name);
        assert_eq!("n#4", iter.next().unwrap().name);
        assert_eq!("n#5", iter.next().unwrap().name);
        assert!(iter.next().is_none());
    }
}
