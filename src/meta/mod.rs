pub mod format;

use self::format::{Format, IntoBoxedFormat};

pub use self::format::Error;

pub trait FormatInto: Format + IntoBoxedFormat {}

impl<T: Format + IntoBoxedFormat> FormatInto for T {}

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
// TODO: Implement Iterator to ease traversing.
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
}

/// Owning evil twin of Meta.
pub struct MetaBuf {
    name: &'static str,
    value: Box<Format>,
}

impl MetaBuf {
    fn new(name: &'static str, value: Box<Format>) -> MetaBuf {
        MetaBuf {
            name: name,
            value: value,
        }
    }
}

impl<'a> From<&'a MetaList<'a>> for Vec<MetaBuf> {
    fn from(val: &'a MetaList<'a>) -> Vec<MetaBuf> {
        let mut result = Vec::with_capacity(32);

        let mut node = val;
        loop {
            for meta in node.meta.iter().rev() {
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

}
