pub use self::grammar::{expression, ParseError};

// TODO: Implement all functionality.
// [x] format_string := <text> [ format <text> ] *
// [x] format := '{' [ argument ] [ ':' format_spec ] '}'
// [ ] argument := integer | identifier
// [ ] format_spec := [[fill]align][sign]['#'][0][width]['.' precision][type]
// [ ] fill := character
// [ ] align := '<' | '^' | '>'
// [ ] sign := '+' | '-'
// [ ] width := count
// [ ] precision := count | '*'
// [ ] type := identifier | ''
// [-] count := parameter | integer
// [-] parameter := integer '$'

const OPENED_BRACE: &'static str = "{";
const CLOSED_BRACE: &'static str = "}";

peg_file! grammar("grammar.peg.rs");

#[derive(Debug, Clone, PartialEq)]
pub enum MetaName {
    Id(usize),
    Name(String),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SeverityType {
    Num,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Timezone {
    Utc,
    Local,
}

/// Enum of alignments which are supported.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Alignment {
    /// The value will be aligned to the left.
    AlignLeft,
    /// The value will be aligned to the right.
    AlignRight,
    /// The value will be aligned in the center.
    AlignCenter,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Portion of the format string which represents the next part to emit.
    Piece(String),
    /// Message with an optional spec.
    Message(Option<FormatSpec>),
    /// Severity formatted as either numeric or string with an optional spec.
    Severity(Option<FormatSpec>, SeverityType),
    /// Timestamp representation with a pattern, timezone and optional spec.
    Timestamp(Option<FormatSpec>, String, Timezone),
    /// Timestamp as a seconds elapsed from Unix epoch with an optional spec.
    TimestampNum(Option<FormatSpec>),
    // Line(Option<Spec>)
    // Module(Option<Spec>)
    // Process(Option<Spec>, ProcessType)
    // Thread(Option<Spec>, ThreadType)
    Meta(MetaName, Option<FormatSpec>),
    // MetaList(Option<Spec>, String[prefix], String[suffix], char[separator], String[pattern], Filter)
}

pub fn parse(pattern: &str) -> Result<Vec<Token>, ParseError> {
    expression(&pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece() {
        let tokens = parse("hello").unwrap();

        assert_eq!(vec![Token::Piece("hello".into())], tokens);
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
    fn meta() {
        let tokens = parse("{hello}").unwrap();

        let expected = vec![Token::Meta(MetaName::Name("hello".into()), None)];
        assert_eq!(expected, tokens);
    }

    // #[test]
    // fn metalist() {
    //     let tokens = parse("{...}").unwrap();
    //
    //     let expected = vec![Token::MetaList(None)];
    //     assert_eq!(expected, tokens);
    // }
}
