use std::io::Write;

use super::Layout;
use super::super::Record;

mod grammar;

use self::grammar::{parse, Align, ParseError, Key, Token};

pub struct PatternLayout {
    tokens: Vec<Token>,
    sevmap: Box<Fn(isize, (char, Option<Align>, Option<usize>), &mut Write) -> Result<(), ::std::io::Error>>,
}

impl PatternLayout {
    pub fn new(pattern: &str) -> Result<PatternLayout, ParseError> {
        PatternLayout::with(pattern, |severity, (fill, align, width), wr| -> Result<(), ::std::io::Error> {
            padded(fill, &align, &width, format!("{}", severity).as_bytes(), wr)
        })
    }

    fn with<F>(pattern: &str, sevmap: F) -> Result<PatternLayout, ParseError>
        where F: Fn(isize, (char, Option<Align>, Option<usize>), &mut Write) -> Result<(), ::std::io::Error> + 'static
    {
        let layout = PatternLayout {
            tokens: parse(pattern)?,
            sevmap: box sevmap,
        };

        Ok(layout)
    }
}

fn padded(fill: char, align: &Option<Align>, width: &Option<usize>, data: &[u8], wr: &mut Write) ->
    Result<(), ::std::io::Error>
{
    let diff = match *width {
        Some(width) if width > data.len() => width - data.len(),
        Some(..) | None => 0,
    };

    let (lpad, rpad) = match *align {
        Some(Align::Left) => (0, diff),
        Some(Align::Right) => (diff, 0),
        Some(Align::Middle) => (diff / 2, diff - diff / 2),
        None => (0, 0),
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

impl Layout for PatternLayout {
    // Errors: Io | KeyNotFound.
    fn format(&mut self, rec: &Record, wr: &mut Write) {
        for token in &self.tokens {
            match *token {
                Token::Literal(ref literal) => {
                    wr.write_all(literal.as_bytes()).unwrap();
                }
                Token::Message(None, None) => {
                    wr.write_all(rec.message().as_bytes()).unwrap();
                }
                Token::Message(align, width) => {
                    padded(' ', &align, &width, rec.message().as_bytes(), wr).unwrap();
                }
                Token::Severity(align, width, ty) => {
                    match ty {
                        'd' => padded(' ', &align, &width, format!("{}", rec.severity()).as_bytes(), wr).unwrap(),
                        's' => (*self.sevmap)(rec.severity(), (' ', align, width), wr).unwrap(),
                        _ => unreachable!(),
                    }
                }
                Token::Placeholder(ref _pattern, Key::Id(..)) => {
                    unimplemented!();
                }
                Token::Placeholder(ref _pattern, Key::Name(ref _name)) => {
                    unimplemented!();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    #[cfg(feature="benchmark")]
    use test::Bencher;

    use Record;
    use layout::Layout;
    use layout::pattern::PatternLayout;

    #[test]
    fn message() {
        let mut layout = PatternLayout::new("[{message}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_message(b: &mut Bencher) {
        let mut layout = PatternLayout::new("message: {message}").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();

        b.iter(|| {
            layout.format(&rec, &mut buf);
            buf.clear();
        });
    }

    #[test]
    fn message_with_spec() {
        let mut layout = PatternLayout::new("[{message:<10}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[value     ]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_width_less_than_length() {
        let mut layout = PatternLayout::new("[{message:<0}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_message_with_spec(b: &mut Bencher) {
        let mut layout = PatternLayout::new("message: {message:<10}").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();

        b.iter(|| {
            layout.format(&rec, &mut buf);
            buf.clear();
        });
    }

    #[test]
    fn severity() {
        // NOTE: No severity mapping provided, layout falls back to the numeric case.
        let mut layout = PatternLayout::new("[{severity}]").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[0]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num() {
        let mut layout = PatternLayout::new("[{severity:d}]").unwrap();

        let rec = Record::new(4, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[4]", from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_severity_with_message(b: &mut Bencher) {
        let mut layout = PatternLayout::new("{severity:d}: {message}").unwrap();

        let rec = Record::new(0, "value");
        let mut buf = Vec::new();

        b.iter(|| {
            layout.format(&rec, &mut buf);
            buf.clear();
        });
    }

    #[test]
    fn severity_with_mapping() {
        let mut layout = PatternLayout::with("[{severity}]", |severity, _spec, wr| {
            assert_eq!(2, severity);

            wr.write_all("DEBUG".as_bytes())
        }).unwrap();

        let rec = Record::new(2, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[DEBUG]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num_with_mapping() {
        let mut layout = PatternLayout::with("[{severity:d}]", |severity, _spec, wr| {
            assert_eq!(2, severity);

            wr.write_all("DEBUG".as_bytes())
        }).unwrap();

        let rec = Record::new(2, "value");
        let mut buf = Vec::new();
        layout.format(&rec, &mut buf);

        assert_eq!("[2]", from_utf8(&buf[..]).unwrap());
    }
}
