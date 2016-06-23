//! Traits and type definition for meta information marshalling.
//!
//! The `blacklog::meta::encode` module contains a number of common things you'll need when dealing
//! with logging meta information (also known as attributes). The most code part of this module is
//! the `Encode` trait that every meta information type should implement to be able properly
//! encoded into bytes.
//! There are common implementations for well-known types, but you are free to extend them for your
//! own types.

use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;

pub type Error = ::std::io::Error;

pub trait Encode : Debug + Send + Sync {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error>;
}

pub trait ToEncodeBox {
    fn to_encode_buf(&self) -> Box<Encode>;
}

pub trait Encoder {
    fn encode_bool(&mut self, value: bool) -> Result<(), Error>;
    fn encode_u64(&mut self, value: u64) -> Result<(), Error>;
    fn encode_str(&mut self, value: &str) -> Result<(), Error>;
}

impl Encode for bool {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_bool(*self)
    }
}

impl ToEncodeBox for bool {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

impl Encode for u64 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_u64(*self)
    }
}

impl ToEncodeBox for u64 {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

impl Encode for &'static str {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(self)
    }
}

impl ToEncodeBox for &'static str {
    fn to_encode_buf(&self) -> Box<Encode> {
        // box self.to_owned()
        box Cow::Borrowed(*self)
    }
}

impl Encode for str {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(self)
    }
}

impl ToEncodeBox for str {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

// TODO: Does it ever works?
impl<'a> Encode for Cow<'a, str> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(self)
    }
}

impl<'a> ToEncodeBox for Cow<'a, str> {
    fn to_encode_buf(&self) -> Box<Encode> {
        unimplemented!()
        // box self.to_owned()
    }
}

impl Encode for String {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), Error> {
        encoder.encode_str(&self[..])
    }
}

impl ToEncodeBox for String {
    fn to_encode_buf(&self) -> Box<Encode> {
        box self.to_owned()
    }
}

impl<W: Write> Encoder for W {
    fn encode_bool(&mut self, value: bool) -> Result<(), Error> {
        write!(self, "{}", value)
    }

    fn encode_u64(&mut self, value: u64) -> Result<(), Error> {
        write!(self, "{}", value)
    }

    fn encode_str(&mut self, value: &str) -> Result<(), Error> {
        write!(self, "{}", value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::Encoder;

    #[test]
    fn encode_true() {
        let mut wr = Vec::new();

        wr.encode_bool(true).unwrap();

        assert_eq!("true".as_bytes(), &wr[..]);
    }
}
