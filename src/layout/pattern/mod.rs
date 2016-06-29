use std::io::Write;

use registry::Config;
use {Format, Formatter, Record, Severity};

use super::{Error, Layout, LayoutFactory};

mod grammar;

use self::grammar::{parse, FormatSpec, ParseError, SeverityType, Timezone, TokenBuf};

pub trait SevMap : Send + Sync {
    fn map(&self, severity: Severity, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
        Result<(), ::std::io::Error>;
}

struct DefaultSevMap;

impl SevMap for DefaultSevMap {
    fn map(&self, severity: Severity, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
        Result<(), ::std::io::Error>
    {
        severity.format(&mut Formatter::new(wr, spec.into()))
    }
}

pub struct PatternLayout<F: SevMap> {
    tokens: Vec<TokenBuf>,
    sevmap: F,
}

impl PatternLayout<DefaultSevMap> {
    pub fn new(pattern: &str) -> Result<PatternLayout<DefaultSevMap>, ParseError> {
        PatternLayout::with(pattern, DefaultSevMap)
    }
}

impl<F: SevMap> PatternLayout<F> {
    fn with(pattern: &str, sevmap: F) -> Result<PatternLayout<F>, ParseError> {
        let layout = PatternLayout {
            tokens: parse(pattern)?.into_iter().map(From::from).collect(),
            sevmap: sevmap,
        };

        Ok(layout)
    }
}

impl<F: SevMap> Layout for PatternLayout<F> {
    fn format(&self, rec: &Record, mut wr: &mut Write) -> Result<(), Error> {
        for token in &self.tokens {
            match *token {
                TokenBuf::Piece(ref piece) => {
                    wr.write_all(piece.as_bytes())?
                }
                TokenBuf::Message(None) => {
                    wr.write_all(rec.message().as_bytes())?
                }
                TokenBuf::Message(Some(spec)) => {
                    rec.message().format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::Severity(None, SeverityType::Num) => {
                    rec.severity().format(&mut Formatter::new(wr, Default::default()))?
                }
                TokenBuf::Severity(None, SeverityType::String) => {
                    self.sevmap.map(rec.severity(), Default::default(), SeverityType::String, wr)?
                }
                TokenBuf::Severity(Some(spec), SeverityType::Num) => {
                    rec.severity().format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::Severity(Some(spec), SeverityType::String) => {
                    // Format all.
                    unimplemented!();
                }
                TokenBuf::TimestampNum(None) => {
                    // Format as seconds (or microseconds) elapsed from Unix epoch.
                    unimplemented!();
                }
                TokenBuf::Timestamp(None, ref pattern, Timezone::Utc) => {
                    // TODO: Replace with write! macro. Measure.
                    wr.write_all(format!("{}", rec.timestamp().format(&pattern)).as_bytes())?
                }
                TokenBuf::Meta(ref name, None) => {
                    let meta = rec.iter().find(|meta| meta.name == name)
                        .ok_or(Error::MetaNotFound)?;

                    meta.value.format(&mut Formatter::new(wr, Default::default()))?;
                }
                TokenBuf::Meta(ref name, Some(spec)) => {
                    let meta = rec.iter().find(|meta| meta.name == name)
                        .ok_or(Error::MetaNotFound)?;

                    meta.value.format(&mut Formatter::new(wr, spec.into()))?;
                }
                TokenBuf::MetaList(None) => {
                    let mut iter = rec.iter();
                    if let Some(meta) = iter.next() {
                        wr.write_all(meta.name.as_bytes())?;
                        write!(wr, ": ")?;
                        meta.value.format(&mut Formatter::new(wr, Default::default()))?;
                    }

                    for meta in iter {
                        write!(wr, ", ")?;
                        wr.write_all(meta.name.as_bytes())?;
                        write!(wr, ": ")?;
                        meta.value.format(&mut Formatter::new(wr, Default::default()))?;
                    }
                }
                _ => unimplemented!(),
            }
        }

        Ok(())
    }
}

pub struct PatternLayoutFactory;

impl LayoutFactory for PatternLayoutFactory {
    fn ty() -> &'static str {
        "pattern"
    }

    fn from(&self, cfg: &Config) -> Result<Box<Layout>, Box<::std::error::Error>> {
        let pattern = cfg.find("pattern")
            .ok_or(r#"field "pattern" is required"#)?
            .as_string()
            .ok_or(r#"field "pattern" must be a string"#)?;
        let res = box PatternLayout::new(pattern)?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::str::from_utf8;

    #[cfg(feature="benchmark")]
    use test::Bencher;

    use {Record, Severity};
    use layout::Layout;
    use layout::pattern::{PatternLayout, SevMap};
    use layout::pattern::grammar::{FormatSpec, SeverityType};
    use meta::format::Alignment;

    // TODO: Seems quite required for other testing modules. Maybe move into `record` module?
    macro_rules! record {
        ($sev:expr, $msg:expr, {$($name:ident: $val:expr,)*}) => {
            Record::new($sev, line!(), module_path!(), format_args!($msg), &$crate::MetaList::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ]))
        };
    }

    #[test]
    fn empty() {
        let layout = PatternLayout::new("").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "", {}).activate(), &mut buf).unwrap();

        assert_eq!("", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn piece() {
        let layout = PatternLayout::new("hello").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "", {}).activate(), &mut buf).unwrap();

        assert_eq!("hello", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn piece_with_braces() {
        let layout = PatternLayout::new("hello {{ world }}").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "", {}).activate(), &mut buf).unwrap();

        assert_eq!("hello { world }", from_utf8(&buf[..]).unwrap());
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
    fn message_with_spec_fill() {
        let layout = PatternLayout::new("[{message:.<10}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[value.....]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_spec_width_less_than_length() {
        let layout = PatternLayout::new("[{message:<0}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_spec_full() {
        let layout = PatternLayout::new("{message:/^6.4}").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(0, "100500", {}).activate(), &mut buf).unwrap();

        assert_eq!("/1005/", from_utf8(&buf[..]).unwrap());
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

        impl SevMap for Mapping {
            fn map(&self, severity: Severity, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
                Result<(), ::std::io::Error>
            {
                assert_eq!(2, severity);
                assert_eq!(' ', spec.fill);
                assert_eq!(Alignment::AlignUnknown, spec.align);
                assert_eq!(0, spec.width);
                assert_eq!(SeverityType::String, ty);
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

        impl SevMap for Mapping {
            fn map(&self, severity: Severity, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
                Result<(), ::std::io::Error>
            {
                assert_eq!(2, severity);
                assert_eq!(' ', spec.fill);
                assert_eq!(Alignment::AlignLeft, spec.align);
                assert_eq!(0, spec.width);
                assert_eq!(SeverityType::Num, ty);
                wr.write_all("DEBUG".as_bytes())
            }
        }

        let layout = PatternLayout::with("[{severity:d}]", Mapping).unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(2, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[2]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num_with_spec() {
        let layout = PatternLayout::new("[{severity:/^3d}]").unwrap();

        let mut buf = Vec::new();
        layout.format(&record!(4, "value", {}).activate(), &mut buf).unwrap();

        assert_eq!("[/4/]", from_utf8(&buf[..]).unwrap());
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
    fn bench_severity(b: &mut Bencher) {
        fn run<'a>(rec: &Record<'a>, b: &mut Bencher) {
            let layout = PatternLayout::new("{severity:d}").unwrap();

            let mut buf = Vec::with_capacity(128);

            b.iter(|| {
                layout.format(&rec, &mut buf).unwrap();
                buf.clear();
            });
        }

        run(&record!(0, "", {}).activate(), b);
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

    #[test]
    fn meta_bool() {
        fn run<'a>(rec: &Record<'a>) {
            let layout = PatternLayout::new("{flag}").unwrap();

            let mut buf = Vec::new();
            layout.format(rec, &mut buf).unwrap();

            assert_eq!("false", from_utf8(&buf[..]).unwrap());
        }

        run(&record!(0, "", {
            flag: false,
        }).activate());
    }

    #[test]
    fn meta_f64_with_spec() {
        fn run<'a>(rec: &Record<'a>) {
            let layout = PatternLayout::new("{pi:/^6.2}").unwrap();

            let mut buf = Vec::new();
            layout.format(rec, &mut buf).unwrap();

            assert_eq!("/3.14/", from_utf8(&buf[..]).unwrap());
        }

        run(&record!(0, "", {
            pi: 3.1415,
        }).activate());
    }

    #[test]
    fn fail_meta_not_found() {
        fn run<'a>(rec: &Record<'a>) {
            let layout = PatternLayout::new("{flag}").unwrap();

            let mut buf = Vec::new();
            assert!(layout.format(rec, &mut buf).is_err());
        }

        run(&record!(0, "", {}).activate());
    }

    #[test]
    fn metalist() {
        fn run<'a>(rec: &Record<'a>) {
            let layout = PatternLayout::new("{...}").unwrap();

            let mut buf = Vec::new();
            layout.format(rec, &mut buf).unwrap();

            assert_eq!("num: 42, name: Vasya", from_utf8(&buf[..]).unwrap());
        }

        run(&record!(0, "value", {
            num: 42,
            name: "Vasya",
        }).activate());
    }
}
