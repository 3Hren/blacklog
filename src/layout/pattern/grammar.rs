pub use self::grammar::{expression, ParseError};

use meta;
use meta::format::Alignment;

const OPENED_BRACE: &'static str = "{";
const CLOSED_BRACE: &'static str = "}";

peg_file! grammar("grammar.peg.rs");

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SeverityType {
    Num,
    String,
}

// TODO: Uncomment.
// #[derive(Debug, Copy, Clone, PartialEq)]
// pub enum ThreadType {
//     Num,
//     String,
// }

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessType {
    Id,
    Name,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Timezone {
    Utc,
    Local,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FormatSpec {
    pub fill: char,
    pub align: Alignment,
    pub flags: u32,
    pub precision: Option<usize>,
    pub width: usize,
}

impl Default for FormatSpec {
    fn default() -> FormatSpec {
        FormatSpec {
            fill: ' ',
            align: Alignment::AlignUnknown,
            flags: 0,
            precision: None,
            width: 0,
        }
    }
}

impl Into<meta::format::FormatSpec> for FormatSpec {
    fn into(self) -> meta::format::FormatSpec {
        meta::format::FormatSpec {
            fill: self.fill,
            align: self.align,
            flags: self.flags,
            precision: self.precision,
            width: self.width,
            ty: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    /// Portion of the format string which represents the next part to emit.
    Piece(&'a str),
    /// Message with an optional spec.
    Message(Option<FormatSpec>),
    /// Severity formatted as either numeric or string with an optional spec.
    Severity(Option<FormatSpec>, SeverityType),
    /// Timestamp representation with a pattern, timezone and optional spec.
    Timestamp(Option<FormatSpec>, String, Timezone),
    /// Timestamp as a seconds elapsed from Unix epoch with an optional spec.
    TimestampNum(Option<FormatSpec>),
    /// The line number on which the logging event was created.
    Line(Option<FormatSpec>),
    /// The module path where the logging event was created.
    Module(Option<FormatSpec>),
    /// Thread id or its name depending on type specified.
    // Thread(Option<FormatSpec>, ThreadType),
    /// Process id (aka PID) or its name depending on type specified.
    Process(Option<FormatSpec>, ProcessType),
    Meta(&'a str, Option<FormatSpec>),
    MetaList(Option<FormatSpec>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenBuf {
    Piece(String),
    Message(Option<FormatSpec>),
    Severity(Option<FormatSpec>, SeverityType),
    Timestamp(Option<FormatSpec>, String, Timezone),
    TimestampNum(Option<FormatSpec>),
    Line(Option<FormatSpec>),
    Module(Option<FormatSpec>),
    // TODO: Thread(Option<FormatSpec>, ThreadType),
    Process(Option<FormatSpec>, ProcessType),
    Meta(String, Option<FormatSpec>),
    MetaList(Option<FormatSpec>),
}

impl<'a> From<Token<'a>> for TokenBuf {
    fn from(val: Token<'a>) -> TokenBuf {
        match val {
            Token::Piece(piece) => TokenBuf::Piece(piece.into()),
            Token::Message(spec) => TokenBuf::Message(spec),
            Token::Severity(spec, ty) => TokenBuf::Severity(spec, ty),
            Token::Timestamp(spec, pattern, tz) => TokenBuf::Timestamp(spec, pattern, tz),
            Token::TimestampNum(spec) => TokenBuf::TimestampNum(spec),
            Token::Line(spec) => TokenBuf::Line(spec),
            Token::Module(spec) => TokenBuf::Module(spec),
            Token::Process(spec, ty) => TokenBuf::Process(spec, ty),
            Token::Meta(name, spec) => TokenBuf::Meta(name.into(), spec),
            Token::MetaList(spec) => TokenBuf::MetaList(spec),
        }
    }
}

pub fn parse(pattern: &str) -> Result<Vec<Token>, ParseError> {
    expression(&pattern)
}

#[cfg(test)]
mod tests {
    use meta::format::Alignment;

    use super::*;

    #[test]
    fn empty() {
        let expected: Vec<Token> = vec![];
        assert_eq!(expected, parse("").unwrap());
    }

    #[test]
    fn piece() {
        let tokens = parse("hello").unwrap();

        assert_eq!(vec![Token::Piece("hello")], tokens);
    }

    #[test]
    fn message() {
        let tokens = parse("{message}").unwrap();

        assert_eq!(vec![Token::Message(None)], tokens);
    }

    #[test]
    fn message_spec() {
        let tokens = parse("{message:.<10.8}").unwrap();

        let spec = FormatSpec {
            fill: '.',
            align: Alignment::AlignLeft,
            flags: 0,
            precision: Some(8),
            width: 10,
        };
        assert_eq!(vec![Token::Message(Some(spec))], tokens);
    }

    #[test]
    fn severity() {
        let tokens = parse("{severity}").unwrap();

        assert_eq!(vec![Token::Severity(None, SeverityType::String)], tokens);
    }

    #[test]
    fn severity_string() {
        let tokens = parse("{severity:s}").unwrap();

        assert_eq!(vec![Token::Severity(None, SeverityType::String)], tokens);
    }

    #[test]
    fn severity_num() {
        let tokens = parse("{severity:d}").unwrap();

        assert_eq!(vec![Token::Severity(None, SeverityType::Num)], tokens);
    }

    #[test]
    fn severity_ext() {
        let tokens = parse("{severity:<10}").unwrap();

        let spec = FormatSpec {
            fill: ' ',
            align: Alignment::AlignLeft,
            flags: 0,
            precision: None,
            width: 10,
        };
        assert_eq!(vec![Token::Severity(Some(spec), SeverityType::String)], tokens);
    }

    #[test]
    fn severity_ext_with_fill() {
        let tokens = parse("{severity:.^16}").unwrap();

        let spec = FormatSpec {
            fill: '.',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: None,
            width: 16,
        };
        assert_eq!(vec![Token::Severity(Some(spec), SeverityType::String)], tokens);
    }

    #[test]
    fn severity_ext_with_fill_num() {
        let tokens = parse("{severity:.^16d}").unwrap();

        let spec = FormatSpec {
            fill: '.',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: None,
            width: 16,
        };
        assert_eq!(vec![Token::Severity(Some(spec), SeverityType::Num)], tokens);
    }

    #[test]
    fn severity_ext_with_fill_string() {
        let tokens = parse("{severity:.^16s}").unwrap();

        let spec = FormatSpec {
            fill: '.',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: None,
            width: 16,
        };
        assert_eq!(vec![Token::Severity(Some(spec), SeverityType::String)], tokens);
    }

    #[test]
    fn severity_with_precision() {
        let tokens = parse("{severity:.1}").unwrap();

        let spec = FormatSpec {
            fill: ' ',
            align: Alignment::AlignLeft,
            flags: 0,
            precision: Some(1),
            width: 0,
        };
        assert_eq!(vec![Token::Severity(Some(spec), SeverityType::String)], tokens);
    }

    #[test]
    fn timestamp() {
        let tokens = parse("{timestamp}").unwrap();

        assert_eq!(vec![Token::Timestamp(None, "%+".into(), Timezone::Utc)], tokens);
    }

    #[test]
    fn timestamp_num() {
        let tokens = parse("{timestamp:d}").unwrap();

        assert_eq!(vec![Token::TimestampNum(None)], tokens);
    }

    #[test]
    fn timestamp_utc() {
        let tokens = parse("{timestamp:s}").unwrap();

        assert_eq!(vec![Token::Timestamp(None, "%+".into(), Timezone::Utc)], tokens);
    }

    #[test]
    fn timestamp_local() {
        let tokens = parse("{timestamp:l}").unwrap();

        assert_eq!(vec![Token::Timestamp(None, "%+".into(), Timezone::Local)], tokens);
    }

    #[test]
    fn timestamp_ext_num() {
        let tokens = parse("{timestamp:^20d}").unwrap();

        let spec = FormatSpec {
            fill: ' ',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: None,
            width: 20,
        };
        assert_eq!(vec![Token::TimestampNum(Some(spec))], tokens);
    }

    #[test]
    fn timestamp_ext_num_with_fill() {
        let tokens = parse("{timestamp:.<d}").unwrap();

        let spec = FormatSpec {
            fill: '.',
            align: Alignment::AlignLeft,
            flags: 0,
            precision: None,
            width: 0,
        };
        assert_eq!(vec![Token::TimestampNum(Some(spec))], tokens);
    }

    #[test]
    fn timestamp_with_pattern_utc() {
        let tokens = parse("{timestamp:{%Y-%m-%d}s}").unwrap();

        assert_eq!(vec![Token::Timestamp(None, "%Y-%m-%d".into(), Timezone::Utc)], tokens);
    }

    #[test]
    fn timestamp_with_pattern_local() {
        let tokens = parse("{timestamp:{%Y-%m-%d}l}").unwrap();

        assert_eq!(vec![Token::Timestamp(None, "%Y-%m-%d".into(), Timezone::Local)], tokens);
    }

    #[test]
    fn timestamp_with_pattern_utc_and_braces() {
        let tokens = parse("{timestamp:{%Y-%m-%d {{T}} %H:%M:%S.%.6f}s}").unwrap();

        let expected = vec![
            Token::Timestamp(None, "%Y-%m-%d {T} %H:%M:%S.%.6f".into(), Timezone::Utc)
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn timestamp_with_pattern_utc_and_braces_limit() {
        let tokens = parse("{timestamp:{{{%Y-%m-%dT%H:%M:%S.%.6f}}}s}").unwrap();

        let expected = vec![
            Token::Timestamp(None, "{%Y-%m-%dT%H:%M:%S.%.6f}".into(), Timezone::Utc)
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn timestamp_ext_with_pattern_and_fill_utc() {
        let tokens = parse("{timestamp:{%Y-%m-%d}.<s}").unwrap();

        let spec = FormatSpec {
            fill: '.',
            align: Alignment::AlignLeft,
            flags: 0,
            precision: None,
            width: 0,
        };
        assert_eq!(vec![Token::Timestamp(Some(spec), "%Y-%m-%d".into(), Timezone::Utc)], tokens);
    }

    #[test]
    fn timestamp_ext_with_pattern_local() {
        let tokens = parse("{timestamp:{%Y-%m-%dT%H:%M:%S.%.6f}>50l}").unwrap();

        let spec = FormatSpec {
            fill: ' ',
            align: Alignment::AlignRight,
            flags: 0,
            precision: None,
            width: 50,
        };
        let exp = vec![
            Token::Timestamp(Some(spec), "%Y-%m-%dT%H:%M:%S.%.6f".into(), Timezone::Local),
        ];
        assert_eq!(exp, tokens);
    }

    #[test]
    fn line() {
        let tokens = parse("{line}").unwrap();

        assert_eq!(vec![Token::Line(None)], tokens);
    }

    #[test]
    fn line_spec() {
        let tokens = parse("{line:/^20}").unwrap();

        let spec = FormatSpec {
            fill: '/',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: None,
            width: 20,
        };
        assert_eq!(vec![Token::Line(Some(spec))], tokens);
    }

    #[test]
    fn module() {
        let tokens = parse("{module}").unwrap();

        assert_eq!(vec![Token::Module(None)], tokens);
    }

    #[test]
    fn module_spec() {
        let tokens = parse("{module:/^20.16}").unwrap();

        let spec = FormatSpec {
            fill: '/',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: Some(16),
            width: 20,
        };
        assert_eq!(vec![Token::Module(Some(spec))], tokens);
    }

    #[test]
    fn process() {
        let tokens = parse("{process}").unwrap();

        assert_eq!(vec![Token::Process(None, ProcessType::Id)], tokens);
    }

    #[test]
    fn process_with_spec() {
        let tokens = parse("{process:/^8d}").unwrap();

        let spec = FormatSpec {
            fill: '/',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: None,
            width: 8,
        };
        assert_eq!(vec![Token::Process(Some(spec), ProcessType::Id)], tokens);
    }

    #[test]
    fn meta() {
        let tokens = parse("{hello}").unwrap();

        let expected = vec![Token::Meta("hello", None)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn meta_spec() {
        let spec = FormatSpec {
            fill: '/',
            align: Alignment::AlignCenter,
            flags: 0,
            precision: Some(2),
            width: 6,
        };
        println!("{pi:/^6.2}", pi=3.1415);
        assert_eq!(vec![Token::Meta("pi", Some(spec))], parse("{pi:/^6.2}").unwrap());
    }

    #[test]
    fn metalist() {
        assert_eq!(vec![Token::MetaList(None)], parse("{...}").unwrap());
    }
}
