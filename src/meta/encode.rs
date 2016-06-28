//! Traits and type definition for meta information marshalling.
//!
//! The `blacklog::meta::encode` module contains a number of common things you'll need when dealing
//! with logging meta information (also known as attributes). The most code part of this module is
//! the `Encode` trait that every meta information type should implement to be able properly
//! encoded into bytes.
//! There are common implementations for well-known types, but you are free to extend them for your
//! own types.
// TODO: Well, now it should be called `format.rs`.

use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;

pub type Error = ::std::io::Error;

// TODO: Rename to `Format`.
pub trait Encode : Debug + Send + Sync {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error>;
}

pub trait ToEncodeBox {
    fn to_encode_buf(&self) -> Box<Encode>;
}

/// Enum of alignments which are supported.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Alignment {
    /// The value will be aligned to the left.
    AlignLeft,
    /// The value will be aligned to the right.
    AlignRight,
    /// The value will be aligned in the center.
    AlignCenter,
    // TODO: Document.
    AlignUnknown,
}

/// Specification for the formatting of an argument in the format string.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FormatSpec {
    /// Optionally specified character to fill alignment with.
    pub fill: char,
    /// Optionally specified alignment.
    pub align: Alignment,
    /// Packed version of various flags provided.
    pub flags: u32,
    /// The integer precision to use.
    ///
    /// For non-numeric types, this can be considered a "maximum width". If the resulting string is
    /// longer than this width, then it is truncated down to this many characters and only those
    /// are emitted.
    ///
    /// For integral types, this is ignored.
    ///
    /// For floating-point types, this indicates how many digits after the decimal point should be
    /// printed.
    pub precision: Option<usize>,
    /// The string width requested for the resulting format.
    pub width: usize,
    // TODO: Additional type information. Document. Optional.
    pub ty: Option<char>,
}

///
pub struct Formatter<'a> {
    // TODO: Do we need one more indirection?
    wr: &'a mut Write,
    spec: Option<FormatSpec>,
}

impl<'a> Formatter<'a> {
    pub fn new(wr: &'a mut Write, spec: Option<FormatSpec>) -> Formatter<'a> {
        Formatter {
            wr: wr,
            spec: spec,
        }
    }

    /// Writes some data directly to the underlying buffer contained within this formatter.
    ///
    /// # Note
    ///
    /// This method does not perform any intermediate formatting.
    pub fn write_str(&mut self, data: &str) -> Result<(), Error> {
        self.wr.write_all(data.as_bytes())
    }

    // With spec.
    pub fn write_i64(&mut self, val: i64) -> Result<(), Error> {
        unimplemented!();
    }

    // for () -> write_str. () | None | null etc + pad.
    // for bool -> write_str + pad.
    // for i8..64,u8..u64 - get spec,
    //   types - None, x, X, b, ?, o.
    //   `#` - 0x 0b 0o
    //   `+` - allowed.
    //   `-` - ignore.
    //   `0` - pad.
    //   `precision` - ignore | error.
    //   `width` - total min width.
    //   pad.
    // for f64,
    //   types - None, e, E.
    //   `#` - not allowed.
    //   `+` - allowed.
    //   `-` - ignored.
    //   `0` - ignored.
    //   `precision` - number of digits after dot.
    //   pad.
    // for str - precision + write + pad.

    // TODO: Getters.

    fn pad(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl Encode for bool {
    fn encode(&self, formatter: &mut Formatter) -> Result<(), Error> {
        match *self {
            true => formatter.write_str("true"),
            false => formatter.write_str("false")
        }
    }
}

impl Encode for u64 {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
        // encoder.encode_u64(*self)
        unimplemented!();
    }
}

impl Encode for f64 {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
        // encoder.encode_f64(*self)
        unimplemented!();
    }
}

impl Encode for &'static str {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
        encoder.write_str(self)
    }
}

impl Encode for str {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
        encoder.write_str(self)
    }
}

// TODO: Does it ever work?
// TODO: Maybe for Cow<'a, T>?
impl<'a> Encode for Cow<'a, str> {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
        encoder.write_str(self)
    }
}

//for T
// 1.prepare string
// 2. pad + align.

impl Encode for String {
    fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
        encoder.write_str(&self[..])
    }
}

impl ToEncodeBox for bool {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

impl ToEncodeBox for u64 {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

impl ToEncodeBox for f64 {
    fn to_encode_buf(&self) -> Box<Encode> {
        box *self
    }
}

impl ToEncodeBox for &'static str {
    fn to_encode_buf(&self) -> Box<Encode> {
        // box self.to_owned()
        box Cow::Borrowed(*self)
    }
}

impl ToEncodeBox for str {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

impl<'a> ToEncodeBox for Cow<'a, str> {
    fn to_encode_buf(&self) -> Box<Encode> {
        unimplemented!()
        // box self.to_owned()
    }
}

impl ToEncodeBox for String {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

// impl<'a, W: Write + 'a> Encoder for W {
//     fn encode_bool(&mut self, value: bool) -> Result<(), Error> {
//         write!(self, "{}", value)
//     }
//
//     fn encode_u64(&mut self, value: u64) -> Result<(), Error> {
//         write!(self, "{}", value)
//     }
//
//     fn encode_f64(&mut self, value: f64) -> Result<(), Error> {
//         write!(self, "{}", value)
//     }
//
//     fn encode_str(&mut self, value: &str) -> Result<(), Error> {
//         write!(self, "{}", value)
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::Formatter;
//
//     #[test]
//     fn encode_true() {
//         let mut wr = Vec::new();
//
//         wr.encode_bool(true).unwrap();
//
//         assert_eq!("true".as_bytes(), &wr[..]);
//     }
//
//     #[test]
//     fn encode_f64() {
//         let mut wr = Vec::new();
//
//         wr.encode_f64(3.1415).unwrap();
//
//         assert_eq!("3.1415".as_bytes(), &wr[..]);
//     }
// }
