use std::io::Write;

use super::{Error, Layout};

use Record;
use Severity;

mod grammar;

use self::grammar::{parse, Align, ParseError, SeverityType, Token};

fn padded(fill: &Option<char>, align: &Option<Align>, width: &Option<usize>, data: &[u8], wr: &mut Write) ->
    Result<(), ::std::io::Error>
{
    let fill = match *fill {
        Some(fill) => fill,
        None => ' ',
    };

    let diff = match *width {
        Some(width) if width > data.len() => width - data.len(),
        Some(..) | None => 0,
    };

    let (lpad, rpad) = match *align {
        Some(Align::Left) | None => (0, diff),
        Some(Align::Right) => (diff, 0),
        Some(Align::Middle) => (diff / 2, diff - diff / 2),
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

pub type SeveritySpec = (Option<char>, Option<Align>, Option<usize>);

pub trait SeverityMapping {
    fn map(&self, severity: Severity, spec: SeveritySpec, wr: &mut Write) ->
        Result<(), ::std::io::Error>;
}

struct DefaultSeverityMapping;

impl SeverityMapping for DefaultSeverityMapping {
    fn map(&self, severity: Severity, (fill, align, width): SeveritySpec, wr: &mut Write) ->
        Result<(), ::std::io::Error>
    {
        padded(&fill, &align, &width, format!("{}", severity).as_bytes(), wr)
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
                Token::Literal(ref literal) => {
                    wr.write_all(literal.as_bytes())?
                }
                Token::Message(None, None, None) => {
                    wr.write_all(rec.message().as_bytes())?
                }
                Token::Message(fill, align, width) => {
                    padded(&fill, &align, &width, rec.message().as_bytes(), wr)?
                }
                Token::Severity(align, width, ty) => {
                    match ty {
                        SeverityType::Num => {
                            padded(&Some(' '), &align, &width, format!("{}", rec.severity()).as_bytes(), wr)?
                        }
                        SeverityType::String => {
                            self.sevmap.map(rec.severity(), (Some(' '), align, width), wr)?
                        }
                    }
                }
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

    use Record;
    use Severity;
    use layout::Layout;
    use layout::pattern::{PatternLayout, SeveritySpec, SeverityMapping};

    #[test]
    fn message() {
        let layout = PatternLayout::new("[{message}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_message(b: &mut Bencher) {
        let layout = PatternLayout::new("message: {message}").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::with_capacity(128);

        b.iter(|| {
            layout.format(&rec, &mut buf).unwrap();
            buf.clear();
        });
    }

    #[test]
    fn message_with_spec() {
        let layout = PatternLayout::new("[{message:<10}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value     ]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_fill() {
        let layout = PatternLayout::new("[{message:.<10}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value.....]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_width_less_than_length() {
        let layout = PatternLayout::new("[{message:<0}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_message_with_spec(b: &mut Bencher) {
        let layout = PatternLayout::new("message: {message:<10}").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::with_capacity(128);

        b.iter(|| {
            layout.format(&rec, &mut buf).unwrap();
            buf.clear();
        });
    }

    #[test]
    fn severity() {
        // NOTE: No severity mapping provided, layout falls back to the numeric case.
        let layout = PatternLayout::new("[{severity}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[0]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num() {
        let layout = PatternLayout::new("[{severity:d}]").unwrap();

        let rec = Record::new(4, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[4]", from_utf8(&buf[..]).unwrap());
    }

    struct Mapping;

    impl SeverityMapping for Mapping {
        fn map(&self, severity: Severity, _spec: SeveritySpec, wr: &mut Write) ->
            Result<(), ::std::io::Error>
        {
            assert_eq!(2, severity);
            wr.write_all("DEBUG".as_bytes())
        }
    }

    #[test]
    fn severity_with_mapping() {
        let layout = PatternLayout::with("[{severity}]", Mapping).unwrap();

        let rec = Record::new(2, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[DEBUG]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num_with_mapping() {
        let layout = PatternLayout::with("[{severity:d}]", Mapping).unwrap();

        let rec = Record::new(2, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[2]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_with_message() {
        let layout = PatternLayout::new("{severity:d}: {message}").unwrap();

        let rec = Record::new(2, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("2: value", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_severity_with_message(b: &mut Bencher) {
        let layout = PatternLayout::new("{severity:d}: {message}").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::with_capacity(128);

        b.iter(|| {
            layout.format(&rec, &mut buf).unwrap();
            buf.clear();
        });
    }

    #[test]
    fn fail_format_small_buffer() {
        let layout = PatternLayout::new("[{message}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = [0u8];

        assert!(layout.format(&rec, &mut &mut buf[..]).is_err());
    }
}
