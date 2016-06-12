use std::io::Write;

use super::Layout;
use super::super::Record;

mod grammar;

use self::grammar::{parse, Align, Key, Spec, Token};

#[derive(Debug)]
enum Error {}

struct PatternLayout {
    tokens: Vec<Token>,
}

impl PatternLayout {
    fn new(pattern: &str) -> Result<PatternLayout, Error> {
        let layout = PatternLayout {
            tokens: parse(pattern).unwrap(),
        };

        Ok(layout)
    }
}

impl Layout for PatternLayout {
    fn format(&mut self, rec: &Record, wr: &mut Write) {
        for token in &self.tokens {
            let data = match *token {
                Token::Literal(ref literal) => {
                    wr.write_all(literal.as_bytes()).unwrap();
                }
                Token::Message(None) => {
                    wr.write_all(rec.message().as_bytes()).unwrap();
                }
                Token::Message(Some(ref spec)) => {
                    let diff = match *spec.width() {
                        Some(width) => width - rec.message().len(),
                        None => 0,
                    };

                    let (lpad, rpad) = match *spec.align() {
                        Some(Align::Left) => (0, diff),
                        Some(Align::Right) => (diff, 0),
                        Some(Align::Middle) => (diff / 2, diff - diff / 2),
                        None => (0, 0),
                    };

                    for _ in 0..lpad {
                        wr.write(&[' ' as u8]).unwrap();
                    }

                    wr.write_all(rec.message().as_bytes()).unwrap();

                    for _ in 0..rpad {
                        wr.write(&[' ' as u8]).unwrap();
                    }
                }
                Token::Placeholder(ref pattern, Key::Id(..)) => {
                    unimplemented!();
                }
                Token::Placeholder(ref pattern, Key::Name(ref name)) => {
                    unimplemented!();
                }
            };
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
}
