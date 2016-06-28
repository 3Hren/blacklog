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
    // TODO: Document.
    AlignUnknown,
}

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment::AlignUnknown
    }
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
    pub fn write_str(&mut self, data: &str) -> Result<(), Error> {
        self.write_all(data.as_bytes())
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        self.wr.write_all(data)
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

pub trait Format {
    fn format(&self, format: &mut Formatter) -> Result<(), Error>;
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

impl Format for f64 {
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        let mut buf = [0; 128];
        let mut cur = Cursor::new(&mut buf[..]);
        match (format.spec.ty, format.spec.precision) {
            (Some('e'), Some(p)) => unimplemented!(),
            (Some('E'), Some(p)) => unimplemented!(),
            (Some('e'), None) => unimplemented!(),
            (Some('E'), None) => unimplemented!(),
            (_, Some(p)) => write!(&mut cur, "{:.*}", p, *self)?,
            (_, None) => write!(&mut cur, "{:}", *self)?,
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
        42i64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("42", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_neg() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        (-42i64).format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("-42", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_max_bin() {
        let mut spec = FormatSpec::default();
        spec.flags = 0b111;
        spec.ty = Some('b');

        let mut buf = Vec::new();
        9223372036854775807.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+0b111111111111111111111111111111111111111111111111111111111111111",
            from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_i64_min_bin() {
        let mut spec = FormatSpec::default();
        spec.flags = 0b111;
        spec.ty = Some('b');

        let mut buf = Vec::new();
        (-9223372036854775808).format(&mut Formatter::new(&mut buf, spec)).unwrap();

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
        42i64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

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
        42i64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

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
        42i64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+0x2a00000", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        3.1415f64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("3.1415", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn format_f64_neg() {
        let spec = FormatSpec::default();

        let mut buf = Vec::new();
        (-3.1415f64).format(&mut Formatter::new(&mut buf, spec)).unwrap();

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
        3.1415f64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

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
        3.1415f64.format(&mut Formatter::new(&mut buf, spec)).unwrap();

        assert_eq!("+00003.142", from_utf8(&buf[..]).unwrap());
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
                42i64.format(&mut format).unwrap();
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
                42i64.format(&mut format).unwrap();
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
                3.1415f64.format(&mut format).unwrap();
            }
            buf.clear();
        });
    }
}
