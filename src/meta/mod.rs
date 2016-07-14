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
    /// Formattable value reference.
    pub value: &'a FormatInto,
}

impl<'a> Meta<'a> {
    /// Constructs a new Meta struct with the given name and value.
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
///
/// This structure represents a link of an intrusive backward linked list with elements on stack.
/// Such exotic data structure was chosen primarily for performance reasons on construction to
/// completely avoid heap allocations.
///
/// Unfortunately due to Rust borrow system it's impossible (at least I didn't find out how) to
/// implement forward linked-list without unsafe code, that's why this list contains only reference
/// to the previous link, which gives an O(N^2) complexity for forward traversal. On the other side
/// usually there aren't so much links (likely 2-3), so this complexity shouldn't hurt much.
#[derive(Debug)]
pub struct MetaLink<'a> {
    /// Position in the linked list.
    id: usize,
    data: &'a [Meta<'a>],
    prev: Option<&'a MetaLink<'a>>,
}

impl<'a> MetaLink<'a> {
    /// Constructs a new link of meta linked list, that acts like a head of the entire list.
    pub fn new(data: &'a [Meta<'a>]) -> MetaLink<'a> {
        MetaLink {
            id: 0,
            data: data,
            prev: None,
        }
    }

    /// Constructs a new link of meta linked list, that is appended to the given one.
    pub fn with_link(data: &'a [Meta<'a>], prev: &'a MetaLink<'a>) -> MetaLink<'a> {
        MetaLink {
            id: 1 + prev.id,
            data: data,
            prev: Some(prev),
        }
    }

    /// Returns a front-to-back Meta iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use blacklog::{Meta, MetaLink};
    ///
    /// let val = "le value";
    /// let meta1 = [
    ///     Meta::new("n#1", &val),
    /// ];
    /// let metalink1 = MetaLink::new(&meta1);
    ///
    /// let meta2 = [
    ///     Meta::new("n#2", &val),
    ///     Meta::new("n#3", &val),
    /// ];
    /// let metalink2 = MetaLink::with_link(&meta2, &metalink1);
    ///
    /// let mut iter = metalink2.iter();
    ///
    /// assert_eq!("n#1", iter.next().unwrap().name);
    /// assert_eq!("n#2", iter.next().unwrap().name);
    /// assert_eq!("n#3", iter.next().unwrap().name);
    /// assert!(iter.next().is_none());
    /// ```
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

impl<'a> From<&'a MetaLink<'a>> for Vec<MetaBuf> {
    fn from(val: &'a MetaLink<'a>) -> Vec<MetaBuf> {
        let mut result = Vec::with_capacity(32);

        // TODO: iter + collect?
        let mut node = val;
        loop {
            for meta in node.data.iter() {
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
        let metalink2 = MetaLink::with_link(&meta2, &metalink1);
        let meta3 = [];
        let metalink3 = MetaLink::with_link(&meta3, &metalink2);

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
        let metalink2 = MetaLink::with_link(&meta2, &metalink1);

        let meta3 = [
            Meta::new("n#5", &val),
            Meta::new("n#6", &val),
            Meta::new("n#7", &val),
        ];
        let metalink3 = MetaLink::with_link(&meta3, &metalink2);

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
        let metalink2 = MetaLink::with_link(&meta2, &metalink1);

        let meta3 = [
            Meta::new("n#5", &val),
            Meta::new("n#6", &val),
            Meta::new("n#7", &val),
        ];
        let metalink3 = MetaLink::with_link(&meta3, &metalink2);

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
        let metalink2 = MetaLink::with_link(&meta2, &metalink1);

        let meta3 = [
            Meta::new("n#3", &val),
            Meta::new("n#4", &val),
            Meta::new("n#5", &val),
        ];
        let metalink3 = MetaLink::with_link(&meta3, &metalink2);

        let mut iter = metalink3.iter();

        assert_eq!("n#1", iter.next().unwrap().name);
        assert_eq!("n#2", iter.next().unwrap().name);
        assert_eq!("n#3", iter.next().unwrap().name);
        assert_eq!("n#4", iter.next().unwrap().name);
        assert_eq!("n#5", iter.next().unwrap().name);
        assert!(iter.next().is_none());
    }
}
