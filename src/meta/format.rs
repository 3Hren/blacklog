//! Traits and type definition for meta information marshalling.
//!
//! This module contains a number of common things you'll need when dealing with logging meta
//! information (also known as attributes). The most code part of this module is the `Format` trait
//! that every meta information type should implement to be able properly encoded into bytes.
//! There are common implementations for well-known types, but you are free to extend them for your
//! own types.

use std::borrow::Cow;
use std::io::{Cursor, Write};

pub type Error = ::std::io::Error;

/// Enum of alignments which are supported.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Alignment {
    /// The value will be aligned to the left.
    AlignLeft,
    /// The value will be aligned to the right.
    AlignRight,
    /// The value will be aligned in the center.
    AlignCenter,
    /// The value will take on a default alignment.
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
    /// Particular argument type.
    pub ty: Option<char>,
}

impl Default for FormatSpec {
    fn default() -> FormatSpec {
        FormatSpec {
            fill: ' ',
            align: Alignment::AlignUnknown,
            flags: 0,
            precision: None,
            width: 0,
            ty: None,
        }
    }
}

/// Represents both where to emit formatting strings to and how they should be formatted. A mutable
/// version of this is passed to all formatting traits.
pub struct Formatter<'a> {
    // TODO: Do we need one more indirection?
    wr: &'a mut Write,
    spec: FormatSpec,
}

impl<'a> Formatter<'a> {
    pub fn new(wr: &'a mut Write, spec: FormatSpec) -> Formatter<'a> {
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
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        self.wr.write_all(data)
    }

    /// This function takes a string slice and emits it to the internal buffer after applying the
    /// relevant formatting flags specified.
    ///
    /// # Flags
    ///
    /// This method looks up the following flags:
    ///
    /// - fill      - what to emit as padding.
    /// - align     - string alignment.
    /// - width     - the minimum width of what to emit.
    /// - precision - the maximum length to emit, the string is truncated if it is longer than
    ///               this length.
    pub fn write_str(&mut self, data: &str) -> Result<(), Error> {
        match *self.precision() {
            None => {
                match self.width() {
                    0 => self.wr.write_all(data.as_bytes()),
                    width => {
                        let pad = width.saturating_sub(data.len());
                        self.with_pad(pad, Alignment::AlignLeft, |format| {
                            format.write_all(data.as_bytes())
                        })
                    }
                }
            }
            Some(prec) => {
                let data = if prec < data.len() {
                    &data[..prec]
                } else {
                    &data
                };

                let pad = self.width().saturating_sub(data.len());
                self.with_pad(pad, Alignment::AlignLeft, |format| {
                    format.write_all(data.as_bytes())
                })
            }
        }
    }

    pub fn fill(&self) -> char {
        self.spec.fill
    }

    pub fn align(&self) -> Alignment {
        self.spec.align
    }

    pub fn width(&self) -> usize {
        self.spec.width
    }

    pub fn precision(&self) -> &Option<usize> {
        &self.spec.precision
    }

    pub fn ty(&self) -> &Option<char> {
        &self.spec.ty
    }

    pub fn sign_plus(&self) -> bool {
        self.spec.flags & (1 << 0) != 0
    }

    pub fn alternate(&self) -> bool {
        self.spec.flags & (1 << 1) != 0
    }

    pub fn sign_aware_zero_pad(&self) -> bool {
        self.spec.flags & (1 << 2) != 0
    }

    fn with_pad<F>(&mut self, pad: usize, align: Alignment, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Formatter) -> Result<(), Error>
    {
        let align = if self.spec.align == Alignment::AlignUnknown {
            align
        } else {
            self.spec.align
        };

        let (lpad, rpad) = match align {
            Alignment::AlignLeft => (0, pad),
            Alignment::AlignRight | Alignment::AlignUnknown => (pad, 0),
            Alignment::AlignCenter => (pad / 2, (pad + 1) / 2),
        };

        let fill = self.spec.fill.encode_utf8();
        let fill = unsafe {
            ::std::str::from_utf8_unchecked(fill.as_slice())
        };

        // TODO: Very slow.
        for _ in 0..lpad {
            self.wr.write_all(fill.as_bytes())?;
        }

        f(self)?;

        // TODO: Very slow too.
        for _ in 0..rpad {
            self.wr.write_all(fill.as_bytes())?;
        }

        Ok(())
    }
}

/// Represents a formattable entity.
///
/// Every meta information type that wishes to be printed into layout should implement this trait.
pub trait Format: Send + Sync {
    /// Formats the value using the given formatter.
    ///
    /// The formatter contains both writer and additional information (also known as spec) that
    /// points to how should it be formatted.
    fn format(&self, format: &mut Formatter) -> Result<(), Error>;
}

impl Format for bool {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        match *self {
            true => format.write_str("true"),
            false => format.write_str("false"),
        }
    }
}

impl Format for isize {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as i64).format(format)
    }
}

impl Format for i8 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as i64).format(format)
    }
}

impl Format for i16 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as i64).format(format)
    }
}

impl Format for i32 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as i64).format(format)
    }
}

impl Format for i64 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        const LOWERCASE: &'static str = "0123456789abcdef";
        const UPPERCASE: &'static str = "0123456789ABCDEF";

        let (base, prefix, charset) = match format.spec.ty {
            Some('x') => (16, "0x", LOWERCASE),
            Some('X') => (16, "0x", UPPERCASE),
            Some('o') => (8,  "0o", LOWERCASE),
            Some('b') => (2,  "0b", LOWERCASE),
            Some(..) | None => (10, "", LOWERCASE),
        };

        let prefix = prefix.as_bytes();
        let charset = charset.as_bytes();

        // Calculate width and do a simple formatting into a fixed-size buffer.
        let mut val = *self;
        let mut buf = ['0' as u8; 1 + 2 + 64];
        let mut pos = buf.len();
        for c in buf.iter_mut().rev() {
            *c = charset[(val % base).abs() as usize];
            val /= base;
            pos -= 1;

            if val == 0 {
                break;
            }
        }

        let buf = &buf[pos..];
        let mut pad = format.spec.width.saturating_sub(buf.len());

        if *self < 0 {
            format.write_all("-".as_bytes())?;
            pad = pad.saturating_sub(1);
        } else if format.sign_plus() {
            format.write_all("+".as_bytes())?;
            pad = pad.saturating_sub(1);
        }

        if format.alternate() {
            format.write_all(prefix)?;
            pad = pad.saturating_sub(prefix.len());
        }

        if format.sign_aware_zero_pad() {
            format.spec.fill = '0';
        }

        format.with_pad(pad, Alignment::AlignRight, |format| {
            format.write_all(buf)
        })
    }
}

impl Format for usize {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as u64).format(format)
    }
}

impl Format for u8 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as u64).format(format)
    }
}

impl Format for u16 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as u64).format(format)
    }
}

impl Format for u32 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as u64).format(format)
    }
}

impl Format for u64 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        const LOWERCASE: &'static str = "0123456789abcdef";
        const UPPERCASE: &'static str = "0123456789ABCDEF";

        let (base, prefix, charset) = match format.spec.ty {
            Some('x') => (16, "0x", LOWERCASE),
            Some('X') => (16, "0x", UPPERCASE),
            Some('o') => (8,  "0o", LOWERCASE),
            Some('b') => (2,  "0b", LOWERCASE),
            Some(..) | None => (10, "", LOWERCASE),
        };

        let prefix = prefix.as_bytes();
        let charset = charset.as_bytes();

        // Calculate width and do a simple formatting into a fixed-size buffer.
        let mut val = *self;
        let mut buf = ['0' as u8; 1 + 2 + 64];
        let mut pos = buf.len();
        for c in buf.iter_mut().rev() {
            *c = charset[(val % base) as usize];
            val /= base;
            pos -= 1;

            if val == 0 {
                break;
            }
        }

        let buf = &buf[pos..];
        let mut pad = format.spec.width.saturating_sub(buf.len());

        if format.sign_plus() {
            format.write_all("+".as_bytes())?;
            pad = pad.saturating_sub(1);
        }

        if format.alternate() {
            format.write_all(prefix)?;
            pad = pad.saturating_sub(prefix.len());
        }

        if format.sign_aware_zero_pad() {
            format.spec.fill = '0';
        }

        format.with_pad(pad, Alignment::AlignRight, |format| {
            format.write_all(buf)
        })
    }
}

impl Format for f32 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        (*self as f64).format(format)
    }
}

impl Format for f64 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        let mut buf = [0; 128];
        let mut cur = Cursor::new(&mut buf[..]);
        match (format.spec.ty, format.spec.precision) {
            (Some('e'), Some(prec)) => write!(&mut cur, "{:.*e}", prec, *self)?,
            (Some('E'), Some(prec)) => write!(&mut cur, "{:.*E}", prec, *self)?,
            (Some('e'), None) => write!(&mut cur, "{:e}", *self)?,
            (Some('E'), None) => write!(&mut cur, "{:E}", *self)?,
            (_, Some(prec)) => write!(&mut cur, "{:.*}", prec, *self)?,
            (_, None) => write!(&mut cur, "{}", *self)?,
        }
        let pos = cur.position() as usize;

        let mut pad = format.spec.width.saturating_sub(pos);

        if format.sign_plus() {
            if *self < 0.0 {
                format.write_all("-".as_bytes())?;
            } else {
                format.write_all("+".as_bytes())?;
            }

            pad = pad.saturating_sub(1);
        }

        if format.sign_aware_zero_pad() {
            format.spec.fill = '0';
        }

        format.with_pad(pad, Alignment::AlignRight, |format| {
            format.write_all(&cur.into_inner()[..pos])
        })
    }
}

impl Format for str {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        format.write_str(self)
    }
}

impl Format for &'static str {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        format.write_str(self)
    }
}

impl Format for String {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        format.write_str(&self[..])
    }
}

impl<'a> Format for Cow<'a, str> {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        format.write_str(self)
    }
}

pub trait FormatInto: Format + IntoBoxedFormat {}

impl<T: Format + IntoBoxedFormat> FormatInto for T {}

/// Extends the formatting trait with an ability of how to make a boxed format, which can be safely
/// sent to another thread in the case of asynchronous logging.
pub trait IntoBoxedFormat: Format {
    /// Wraps itself into a boxed format, usually by cloning.
    fn to_boxed_format(&self) -> Box<FormatInto>;
}

impl IntoBoxedFormat for bool {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for usize {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for u8 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for u16 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for u32 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for u64 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for isize {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for i8 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for i16 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for i32 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for i64 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for f32 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for f64 {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box *self
    }
}

impl IntoBoxedFormat for &'static str {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box Cow::Borrowed(*self)
    }
}

impl<'a> IntoBoxedFormat for Cow<'a, str> {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box self.clone().into_owned()
    }
}

impl IntoBoxedFormat for String {
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box self.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;

    #[test]
    fn format_spec_default() {
        let spec = FormatSpec::default();

        assert_eq!(' ', spec.fill);
        assert_eq!(Alignment::AlignUnknown, spec.align);
        assert_eq!(0, spec.flags);
        assert_eq!(None, spec.precision);
        assert_eq!(0, spec.width);
        assert_eq!(None, spec.ty);
    }

    #[test]
    fn format_i64() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        let val = 42i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("42", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_neg() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        let val = -42i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("-42", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_max_bin() {
        let mut spec = FormatSpec::default();
        spec.flags = 0b111;
        spec.ty = Some('b');

        let mut buf = Vec::new();
        let val = 9223372036854775807i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+0b111111111111111111111111111111111111111111111111111111111111111",
            from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_min_bin() {
        let mut spec = FormatSpec::default();
        spec.flags = 0b111;
        spec.ty = Some('b');

        let mut buf = Vec::new();
        let val = -9223372036854775808i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("-0b1000000000000000000000000000000000000000000000000000000000000000",
            from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_spec() {
        let spec = FormatSpec {
            fill: '/',                     // Check.
            align: Alignment::AlignCenter, // Check.
            flags: 0,                      // Not here.
            precision: None,               // Ignored.
            width: 10,                     // Check.
            ty: None,                      // Not here.
        };

        let mut buf = Vec::new();
        let val = 42i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("////42////", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_full_spec() {
        let spec = FormatSpec {
            fill: ' ',                     // Ignored because of `0` flag.
            align: Alignment::AlignRight,  // Check.
            flags: 0b111,                  // Check: `+` | `#` | `0`.
            precision: None,               // Ignored.
            width: 10,                     // Check.
            ty: Some('x'),                 // Check.
        };

        let mut buf = Vec::new();
        let val = 42i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+0x000002a", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_full_spec_left_aligned() {
        let spec = FormatSpec {
            fill: ' ',
            align: Alignment::AlignLeft,
            flags: 0b111,
            precision: None,
            width: 10,
            ty: Some('x'),
        };

        let mut buf = Vec::new();
        let val = 42i64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+0x2a00000", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i32() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        let val = 42i32;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("42", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_u64_max_bin() {
        let mut spec = FormatSpec::default();
        spec.flags = 0b111;
        spec.ty = Some('b');

        let mut buf = Vec::new();
        let val = 18446744073709551615u64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+0b1111111111111111111111111111111111111111111111111111111111111111",
            from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        let val = 3.1415f64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("3.1415", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64_neg() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        let val = -3.1415f64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("-3.1415", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64_with_spec() {
        let mut spec = FormatSpec::default();
        spec.align = Alignment::AlignLeft;
        spec.flags = 0b111;
        spec.precision = Some(3);
        spec.width = 10;

        let mut buf = Vec::new();
        let val = 3.1415f64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+3.1420000", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64_with_spec_right_aligned() {
        let mut spec = FormatSpec::default();
        spec.align = Alignment::AlignRight;
        spec.flags = 0b111;
        spec.precision = Some(3);
        spec.width = 10;

        let mut buf = Vec::new();
        let val = 3.1415f64;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+00003.142", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64_with_spec_exp() {
        let mut spec = FormatSpec::default();
        spec.ty = Some('e');

        let mut buf = Vec::new();
        let val = 100500.0;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("1.005e5", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64_with_spec_exp_and_prec() {
        let mut spec = FormatSpec::default();
        spec.precision = Some(4);
        spec.ty = Some('E');

        let mut buf = Vec::new();
        let val = 100500.0;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("1.0050E5", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f32_spec() {
        let mut spec = FormatSpec::default();
        spec.precision = Some(2);

        let mut buf = Vec::new();
        let val = 3.1415f32;
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("3.14", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_str() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        let val = "le message";
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("le message", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_str_with_spec() {
        let mut spec = FormatSpec::default();
        spec.fill = '/';
        spec.align = Alignment::AlignCenter;
        spec.width = 12;

        let mut buf = Vec::new();
        let val = "le message";
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        println!("{:/^12}", "le message");
        assert_eq!("/le message/", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_str_with_spec_with_precision() {
        let mut spec = FormatSpec::default();
        spec.fill = '/';
        spec.align = Alignment::AlignCenter;
        spec.width = 10;
        spec.precision = Some(8);

        let mut buf = Vec::new();
        let val = "le message";
        val.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("/le messa/", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_bool() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        true.format(&mut Formatter::new(&mut buf, spec)).unwrap();
        buf.push(' ' as u8);
        false.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("true false", from_utf8(&buf[..]).unwrap());
    }
}

#[cfg(feature="benchmark")]
mod bench {
    use test::Bencher;

    use super::*;

    #[bench]
    fn bench_format_i64(b: &mut Bencher) {
        let spec = FormatSpec::default();

        let mut buf = Vec::with_capacity(64);

        b.iter(|| {
            {
                let mut format = Formatter::new(&mut buf, spec);
                let val = 42i64;
                val.format(&mut format).unwrap();
            }
            buf.clear();
        });
    }

    #[bench]
    fn bench_format_i64_spec(b: &mut Bencher) {
        let spec = FormatSpec {
            fill: '/',                     // Check.
            align: Alignment::AlignCenter, // Check.
            flags: 0,                      // Not here.
            precision: None,               // Ignored.
            width: 10,                     // Check.
            ty: None,                      // Not here.
        };

        let mut buf = Vec::with_capacity(64);

        b.iter(|| {
            {
                let mut format = Formatter::new(&mut buf, spec);
                let val = 42i64;
                val.format(&mut format).unwrap();
            }
            buf.clear();
        });
    }

    #[bench]
    fn bench_format_f64(b: &mut Bencher) {
        let spec = FormatSpec::default();

        let mut buf = Vec::with_capacity(64);

        b.iter(|| {
            {
                let mut format = Formatter::new(&mut buf, spec);
                let val = 3.1415f64;
                val.format(&mut format).unwrap();
            }
            buf.clear();
        });
    }

    #[bench]
    fn bench_format_str(b: &mut Bencher) {
        let spec = FormatSpec::default();

        let mut buf = Vec::with_capacity(64);

        b.iter(|| {
            {
                let mut format = Formatter::new(&mut buf, spec);
                let val = "le message";
                val.format(&mut format).unwrap();
            }
            buf.clear();
        });
    }
}
