use std::io::Write;

use super::{Error, Layout};

use Record;
use Severity;

mod grammar;

use self::grammar::{parse, Alignment, FormatSpec, ParseError, SeverityType, TimestampType, Token};

fn padded(fill: char, align: Alignment, width: usize, data: &[u8], wr: &mut Write) ->
    Result<(), ::std::io::Error>
{
    let width = width as usize;
    let diff = if width > data.len() {
        width - data.len()
    } else {
        0
    };

    let (lpad, rpad) = match align {
        Alignment::AlignLeft => (0, diff),
        Alignment::AlignRight => (diff, 0),
        Alignment::AlignCenter => (diff / 2, diff - diff / 2),
    };

    for _ in 0..lpad {
        wr.write(&[fill as u8])?;
    }

    wr.write_all(data)?;

    for _ in 0..rpad {
        wr.write(&[fill as u8])?;
    }

    Ok(())
}

pub trait SeverityMapping {
    fn map(&self, severity: Severity, fill: char, align: Alignment, width: usize, wr: &mut Write) ->
        Result<(), ::std::io::Error>;
}

struct DefaultSeverityMapping;

impl SeverityMapping for DefaultSeverityMapping {
    fn map(&self, severity: Severity, fill: char, align: Alignment, width: usize, wr: &mut Write) ->
        Result<(), ::std::io::Error>
    {
        // TODO: Try transmute.
        padded(fill, align, width, format!("{}", severity).as_bytes(), wr)
    }
}

pub struct PatternLayout<F: SeverityMapping> {
    tokens: Vec<Token>,
    sevmap: F,
}

impl PatternLayout<DefaultSeverityMapping> {
    pub fn new(pattern: &str) -> Result<PatternLayout<DefaultSeverityMapping>, ParseError> {
        PatternLayout::with(pattern, DefaultSeverityMapping)
    }
}

impl<F: SeverityMapping> PatternLayout<F> {
    fn with(pattern: &str, sevmap: F) -> Result<PatternLayout<F>, ParseError> {
        let layout = PatternLayout {
            tokens: parse(pattern)?,
            sevmap: sevmap,
        };

        Ok(layout)
    }
}

impl<F: SeverityMapping> Layout for PatternLayout<F> {
    fn format(&self, rec: &Record, wr: &mut Write) -> Result<(), Error> {
        for token in &self.tokens {
            match *token {
                Token::Piece(ref piece) => {
                    wr.write_all(piece.as_bytes())?
                }
                Token::Message(None) => {
                    wr.write_all(rec.message().as_bytes())?
                }
                Token::Message(Some(spec)) => {
                    padded(spec.fill, spec.align, spec.width, rec.message().as_bytes(), wr)?
                }
                Token::Severity(None, SeverityType::Num) => {
                    wr.write_all(format!("{}", rec.severity()).as_bytes())?
                }
                Token::Severity(None, SeverityType::String) => {
                    self.sevmap.map(rec.severity(), ' ', Alignment::AlignLeft, 0, wr)?
                }
                Token::Severity(Some(spec), SeverityType::Num) => {
                    // Format all.
                    unimplemented!();
                }
                Token::Severity(Some(spec), SeverityType::String) => {
                    // Format all.
                    unimplemented!();
                }
                Token::Timestamp { ty: TimestampType::Utc(ref pattern) } =>
                    wr.write_all(format!("{}", rec.timestamp().format(&pattern)).as_bytes())?,
                _ => unimplemented!(),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::str::from_utf8;

    #[cfg(feature="benchmark")]
    use test::Bencher;

    use {MetaList, Record, Severity};
    use layout::Layout;
    use layout::pattern::{PatternLayout, SeverityMapping};
    use layout::pattern::grammar::Alignment;

    // TODO: Seems quite required for other testing modules. Maybe move into `record` module?
    macro_rules! record {
        ($sev:expr, $msg:expr, {$($name:ident: $val:expr,)*}) => {
            Record::new($sev, line!(), module_path!(), format_args!($msg), &$crate::MetaList::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ]))
        };
    }

    #[test]
    fn piece() {
        let layout = PatternLayout::new("1234567890").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "", {}).activate(), &mut buf).unwrap();

        assert_eq!("1234567890", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn piece_with_braces() {
        let layout = PatternLayout::new("123{{abc}}456").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "", {}).activate(), &mut buf).unwrap();

        assert_eq!("123{abc}456", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message() {
        let layout = PatternLayout::new("message: {message}").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("message: value", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_message(b: &mut Bencher) {
        fn run<'a>(rec: &Record<'a>, b: &mut Bencher) {
            let layout = PatternLayout::new("message: {message}").unwrap();

            let mut buf = Vec::with_capacity(128);
            b.iter(|| {
                layout.format(rec, &mut buf).unwrap();
                buf.clear();
            });
        };

        run(&record!(0, "value", {}).activate(), b);
    }

    #[test]
    fn message_with_spec() {
        let layout = PatternLayout::new("[{message:<10}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[value     ]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_fill() {
        let layout = PatternLayout::new("[{message:.<10}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[value.....]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_width_less_than_length() {
        let layout = PatternLayout::new("[{message:<0}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_message_with_spec(b: &mut Bencher) {
        fn run<'a>(rec: &Record<'a>, b: &mut Bencher) {
            let layout = PatternLayout::new("message: {message:<10}").unwrap();

            let mut buf = Vec::with_capacity(128);

            b.iter(|| {
                layout.format(&rec, &mut buf).unwrap();
                buf.clear();
            });
        }

        run(&record!(0, "value", {}).activate(), b);
    }

    #[test]
    fn severity() {
        // NOTE: No severity mapping provided, layout falls back to the numeric case.
        let layout = PatternLayout::new("[{severity}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[0]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num() {
        let layout = PatternLayout::new("[{severity:d}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(4, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[4]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_with_mapping() {
        struct Mapping;

        impl SeverityMapping for Mapping {
            fn map(&self, severity: Severity, fill: char, align: Alignment, width: usize, wr: &mut Write) ->
                Result<(), ::std::io::Error>
            {
                assert_eq!(' ', fill);
                assert_eq!(Alignment::AlignLeft, align);
                assert_eq!(0, width);
                assert_eq!(2, severity);
                wr.write_all("DEBUG".as_bytes())
            }
        }

        let layout = PatternLayout::with("[{severity}]", Mapping).unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(2, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[DEBUG]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num_with_mapping() {
        struct Mapping;

        impl SeverityMapping for Mapping {
            fn map(&self, severity: Severity, fill: char, align: Alignment, width: usize, wr: &mut Write) ->
                Result<(), ::std::io::Error>
            {
                assert_eq!(' ', fill);
                assert_eq!(Alignment::AlignLeft, align);
                assert_eq!(0, width);
                assert_eq!(2, severity);
                wr.write_all("DEBUG".as_bytes())
            }
        }

        let layout = PatternLayout::with("[{severity:d}]", Mapping).unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(2, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[2]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_with_message() {
        let layout = PatternLayout::new("{severity:d}: {message}").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(2, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("2: value", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_severity_with_message(b: &mut Bencher) {
        fn run<'a>(rec: &Record<'a>, b: &mut Bencher) {
            let layout = PatternLayout::new("{severity:d}: {message}").unwrap();

            let mut buf = Vec::with_capacity(128);

            b.iter(|| {
                layout.format(&rec, &mut buf).unwrap();
                buf.clear();
            });
        }

        run(&record!(0, "value", {}).activate(), b);
    }

    #[test]
    fn fail_format_small_buffer() {
        let layout = PatternLayout::new("[{message}]").unwrap();

        let mut buf = [0u8];

        assert!(layout.format(&record!(0, "value", {}).activate(), &mut &mut buf[..]).is_err());
    }

    #[test]
    fn timestamp() {
        fn run<'a>(rec: &Record<'a>) {
            // NOTE: By default %+ pattern is used.
            let layout = PatternLayout::new("{timestamp}").unwrap();

            let mut buf = Vec::new();
            layout.format(rec, &mut buf).unwrap();

            assert_eq!(format!("{}", rec.timestamp().format("%+")), from_utf8(&buf[..]).unwrap());
        }

        run(&record!(0, "value", {}).activate());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_timestamp(b: &mut Bencher) {
        fn run<'a>(rec: &Record<'a>, b: &mut Bencher) {
            let layout = PatternLayout::new("{timestamp}").unwrap();

            let mut buf = Vec::with_capacity(128);

            b.iter(|| {
                layout.format(&rec, &mut buf).unwrap();
                buf.clear();
            });
        }

        run(&record!(2, "value", {}).activate(), b);
    }
}
