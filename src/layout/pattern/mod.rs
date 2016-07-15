use std::error;
use std::io::Write;

use chrono::Timelike;
use chrono::offset::local::Local;

use {Format, Formatter, Record, Registry};
use factory::Factory;
use registry::Config;

use super::{Error, Layout};

mod grammar;

use self::grammar::{parse, FormatSpec, ParseError, SeverityType, Timezone, TokenBuf};

pub trait SevMap: Send + Sync {
    fn map(&self, rec: &Record, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
        Result<(), ::std::io::Error>;
}

pub struct DefaultSevMap;

impl SevMap for DefaultSevMap {
    fn map(&self, rec: &Record, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
        Result<(), ::std::io::Error>
    {
        let sev = rec.severity();

        match ty {
            SeverityType::Num => {
                sev.format(&mut Formatter::new(wr, spec.into()))
            }
            SeverityType::String => {
                rec.severity_format()(sev, &mut Formatter::new(wr, spec.into()))
            }
        }
    }
}

pub struct PatternLayout<F: SevMap=DefaultSevMap> {
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
                    self.sevmap.map(rec, Default::default(), SeverityType::String, wr)?
                }
                TokenBuf::Severity(Some(spec), SeverityType::Num) => {
                    rec.severity().format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::Severity(Some(spec), SeverityType::String) => {
                    self.sevmap.map(rec, spec, SeverityType::String, wr)?
                }
                TokenBuf::Timestamp(None, ref pattern, Timezone::Utc) => {
                    write!(wr, "{}", rec.datetime().format(&pattern))?
                }
                TokenBuf::Timestamp(None, ref pattern, Timezone::Local) => {
                    write!(wr, "{}", rec.datetime().with_timezone(&Local).format(&pattern))?
                }
                TokenBuf::Timestamp(Some(spec), ref pattern, timezone) => {
                    let tokens = match timezone {
                        Timezone::Utc => rec.datetime().format(&pattern),
                        Timezone::Local => rec.datetime().with_timezone(&Local).format(&pattern),
                    };

                    format!("{}", tokens)
                        .format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::TimestampNum(None) => {
                    let datetime = rec.datetime();
                    let timestamp = datetime.timestamp();
                    let total = timestamp * 1000000 + datetime.nanosecond() as i64 / 1000;

                    total.format(&mut Formatter::new(wr, Default::default()))?
                }
                TokenBuf::TimestampNum(Some(spec)) => {
                    let datetime = rec.datetime();
                    let timestamp = datetime.timestamp();
                    let total = timestamp * 1000000 + datetime.nanosecond() as i64 / 1000;

                    total.format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::Line(None) => {
                    rec.line().format(&mut Formatter::new(wr, Default::default()))?
                }
                TokenBuf::Line(Some(spec)) => {
                    rec.line().format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::Module(None) => {
                    wr.write_all(rec.module().as_bytes())?
                }
                TokenBuf::Module(Some(spec)) => {
                    rec.module().format(&mut Formatter::new(wr, spec.into()))?
                }
                TokenBuf::Process(None, _ty) => {
                    unimplemented!();
                }
                TokenBuf::Process(Some(_spec), _ty) => {
                    unimplemented!();
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
                TokenBuf::MetaList(Some(_spec)) => {
                    unimplemented!();
                }
            }
        }

        Ok(())
    }
}

impl<F: SevMap> Factory for PatternLayout<F> {
    type Item = Layout;

    fn ty() -> &'static str {
        "pattern"
    }

    fn from(cfg: &Config, _registry: &Registry) -> Result<Box<Layout>, Box<error::Error>> {
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

    use chrono::Timelike;
    use chrono::offset::local::Local;

    #[cfg(feature="benchmark")]
    use test::Bencher;

    use {Meta, MetaLink, Record};
    use layout::Layout;
    use layout::pattern::{PatternLayout, SevMap};
    use layout::pattern::grammar::{FormatSpec, SeverityType};
    use meta::format::Alignment;

    // TODO: Seems quite required for other testing modules. Maybe move into `record` module?
    macro_rules! record {
        ($sev:expr, {$($name:ident: $val:expr,)*}) => {
            Record::new($sev, line!(), module_path!(), &$crate::MetaLink::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ]))
        };
    }

    #[test]
    fn empty() {
        let layout = PatternLayout::new("").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn piece() {
        let layout = PatternLayout::new("hello").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("hello", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn piece_with_braces() {
        let layout = PatternLayout::new("hello {{ world }}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("hello { world }", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message() {
        let layout = PatternLayout::new("message: {message}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        layout.format(&rec, &mut buf).unwrap();

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

        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        run(&rec, b);
    }

    #[test]
    fn message_with_spec() {
        let layout = PatternLayout::new("[{message:<10}]").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value     ]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_spec_fill() {
        let layout = PatternLayout::new("[{message:.<10}]").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value.....]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_spec_width_less_than_length() {
        let layout = PatternLayout::new("[{message:<0}]").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[value]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn message_with_spec_full() {
        let layout = PatternLayout::new("{message:/^6.4}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("100500"));
        layout.format(&rec, &mut buf).unwrap();

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

        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        run(&rec, b);
    }

    #[test]
    fn severity() {
        // NOTE: No severity mapping provided, layout falls back to the numeric case.
        let layout = PatternLayout::new("[{severity}]").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[0]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num() {
        let layout = PatternLayout::new("[{severity:d}]").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(4, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[4]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_with_mapping() {
        struct Mapping;

        impl SevMap for Mapping {
            fn map(&self, rec: &Record, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
                Result<(), ::std::io::Error>
            {
                let sev = rec.severity();
                assert_eq!(2, sev);
                assert_eq!(' ', spec.fill);
                assert_eq!(Alignment::AlignUnknown, spec.align);
                assert_eq!(0, spec.width);
                assert_eq!(SeverityType::String, ty);
                wr.write_all("DEBUG".as_bytes())
            }
        }

        let layout = PatternLayout::with("[{severity}]", Mapping).unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(2, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[DEBUG]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num_with_mapping() {
        struct Mapping;

        impl SevMap for Mapping {
            fn map(&self, rec: &Record, spec: FormatSpec, ty: SeverityType, wr: &mut Write) ->
                Result<(), ::std::io::Error>
            {
                let sev = rec.severity();
                assert_eq!(2, sev);
                assert_eq!(' ', spec.fill);
                assert_eq!(Alignment::AlignLeft, spec.align);
                assert_eq!(0, spec.width);
                assert_eq!(SeverityType::Num, ty);
                wr.write_all("DEBUG".as_bytes())
            }
        }

        let layout = PatternLayout::with("[{severity:d}]", Mapping).unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(2, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[2]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_num_with_spec() {
        let layout = PatternLayout::new("[{severity:/^3d}]").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(4, 0, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("[/4/]", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn severity_with_message() {
        let layout = PatternLayout::new("{severity:d}: {message}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(2, 0, "", &metalink);
        rec.activate(format_args!("value"));
        layout.format(&rec, &mut buf).unwrap();

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

        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, "", &metalink);
        run(&rec, b);
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

        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        run(&rec, b);
    }

    #[test]
    fn fail_format_small_buffer() {
        let layout = PatternLayout::new("[{message}]").unwrap();

        let mut buf = [0u8];

        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!("value"));
        assert!(layout.format(&rec, &mut &mut buf[..]).is_err());
    }

    #[test]
    fn timestamp() {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!(""));

        // NOTE: By default %+ pattern is used.
        let layout = PatternLayout::new("{timestamp}").unwrap();

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!(format!("{}", rec.datetime().format("%+")), from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn timestamp_local() {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!(""));

        // NOTE: By default %+ pattern is used.
        let layout = PatternLayout::new("{timestamp:l}").unwrap();

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!(format!("{}", rec.datetime().with_timezone(&Local).format("%+")),
            from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn timestamp_num() {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!(""));

        let layout = PatternLayout::new("{timestamp:d}").unwrap();

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        let datetime = rec.datetime();
        let timestamp = datetime.timestamp();
        let value = timestamp * 1000000 + datetime.nanosecond() as i64 / 1000;
        assert_eq!(format!("{}", value), from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn timestamp_with_spec() {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!(""));

        // NOTE: By default %+ pattern is used.
        let layout = PatternLayout::new("{timestamp:/^6.4s}").unwrap();

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!(format!("/{}/", rec.datetime().format("%Y")), from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn timestamp_local_with_spec() {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!(""));

        let layout = PatternLayout::new("{timestamp:{%H:%M}/^4.2l}").unwrap();

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!(format!("/{}/", rec.datetime().with_timezone(&Local).format("%H")),
            from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn timestamp_num_with_spec() {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(0, 0, "", &metalink);
        rec.activate(format_args!(""));

        let layout = PatternLayout::new("{timestamp:/^18d}").unwrap();

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        let datetime = rec.datetime();
        let timestamp = datetime.timestamp();
        let value = timestamp * 1000000 + datetime.nanosecond() as i64 / 1000;
        assert_eq!(format!("/{}/", value), from_utf8(&buf[..]).unwrap());
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_timestamp(b: &mut Bencher) {
        let metalink = MetaLink::new(&[]);
        let mut rec = Record::new(2, 0, "", &metalink);
        rec.activate(format_args!(""));

        let layout = PatternLayout::new("{timestamp}").unwrap();

        let mut buf = Vec::with_capacity(128);

        b.iter(|| {
            layout.format(&rec, &mut buf).unwrap();
            buf.clear();
        });
    }

    #[test]
    fn meta_bool() {
        fn run<'a>(rec: &Record<'a>) {
            let layout = PatternLayout::new("{flag}").unwrap();

            let mut buf = Vec::new();
            layout.format(rec, &mut buf).unwrap();

            assert_eq!("false", from_utf8(&buf[..]).unwrap());
        }

        let val = false;
        let meta = [
            Meta::new("flag", &val)
        ];
        let metalink = MetaLink::new(&meta);
        let rec = Record::new(0, 0, "", &metalink);
        run(&rec);
    }

    #[test]
    fn meta_f64_with_spec() {
        fn run<'a>(rec: &Record<'a>) {
            let layout = PatternLayout::new("{pi:/^6.2}").unwrap();

            let mut buf = Vec::new();
            layout.format(rec, &mut buf).unwrap();

            assert_eq!("/3.14/", from_utf8(&buf[..]).unwrap());
        }

        let val = 3.1415;
        let meta = [
            Meta::new("pi", &val)
        ];
        let metalink = MetaLink::new(&meta);
        let rec = Record::new(0, 0, "", &metalink);
        run(&rec);
    }

    #[test]
    fn fail_meta_not_found() {
        let layout = PatternLayout::new("{flag}").unwrap();

        let meta = [];
        let metalink = MetaLink::new(&meta);
        let rec = Record::new(0, 0, "", &metalink);

        let mut buf = Vec::new();
        assert!(layout.format(&rec, &mut buf).is_err());
    }

    #[test]
    fn metalist() {
        let layout = PatternLayout::new("{...}").unwrap();

        let v1 = 42;
        let v2 = "Vasya";
        let meta = [
            Meta::new("num", &v1),
            Meta::new("name", &v2),
        ];
        let metalink = MetaLink::new(&meta);
        let rec = Record::new(0, 0, "", &metalink);

        let mut buf = Vec::new();
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("num: 42, name: Vasya", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn module() {
        let layout = PatternLayout::new("{module}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, module_path!(), &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("blacklog::layout::pattern::tests", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn module_with_spec() {
        let layout = PatternLayout::new("{module:/^14.12}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 0, module_path!(), &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("/blacklog::la/", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn line() {
        let layout = PatternLayout::new("{line}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 666, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("666", from_utf8(&buf[..]).unwrap());
    }

    #[test]
    fn line_with_spec() {
        let layout = PatternLayout::new("{line:/^5}").unwrap();

        let mut buf = Vec::new();
        let metalink = MetaLink::new(&[]);
        let rec = Record::new(0, 555, "", &metalink);
        layout.format(&rec, &mut buf).unwrap();

        assert_eq!("/555/", from_utf8(&buf[..]).unwrap());
    }
}
