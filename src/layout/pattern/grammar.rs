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

peg! grammar(r#"
use super::{Align, Key, SeverityType, TimestampType, Timezone, Token, OPENED_BRACE, CLOSED_BRACE};

#[pub]
expression -> Vec<Token>
    = (format / text)+
text -> Token
    = "{{" { Token::Literal(OPENED_BRACE.into()) }
    / "}}" { Token::Literal(CLOSED_BRACE.into()) }
    / [^{}]+ { Token::Literal(match_str.into()) }
format -> Token
    = "{" "message" "}" { Token::Message }
    / "{" "message:" fill:fill? align:align? width:width? "}" {
        Token::MessageExt {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
        }
    }
    / "{" "severity" "}" { Token::Severity { ty: SeverityType::String } }
    / "{" "severity:" "s}" { Token::Severity { ty: SeverityType::String } }
    / "{" "severity:" "d}" { Token::Severity { ty: SeverityType::Num } }
    / "{" "severity:" fill:fill? align:align? width:width? ty:sty? "}" {
        Token::SeverityExt {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
            ty: ty.unwrap_or(SeverityType::String),
        }
    }
    / "{" "timestamp" "}" { Token::Timestamp { ty: TimestampType::Utc("%+".into()) } }
    / "{" "timestamp:" "d}" { Token::Timestamp { ty: TimestampType::Num } }
    / "{" "timestamp:" fill:fill? align:align? width:width? "d}" {
        Token::TimestampExt {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
            ty: TimestampType::Num,
        }
    }
    / "{" "timestamp:" pattern:strftime? tz:tz "}" {
        match tz {
            Timezone::Utc =>
                Token::Timestamp { ty: TimestampType::Utc(pattern.unwrap_or("%+".into())) },
            Timezone::Local =>
                Token::Timestamp { ty: TimestampType::Local(pattern.unwrap_or("%+".into())) },
        }
    }
    / "{" "timestamp:" pattern:strftime? fill:fill? align:align? width:width? tz:tz "}" {
        let ty = match tz {
            Timezone::Utc => TimestampType::Utc(pattern.unwrap_or("%+".into())),
            Timezone::Local => TimestampType::Local(pattern.unwrap_or("%+".into())),
        };

        Token::TimestampExt {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
            ty: ty,
        }
    }
    / "{" key:name "}" { Token::Placeholder(match_str[1..match_str.len() - 1].into(), key) }
fill -> char
    = . &align { match_str.chars().next().unwrap() }
align -> Align
    = "<" { Align::Left }
    / ">" { Align::Right }
    / "^" { Align::Middle }
width -> usize
    = [0-9]+ { match_str.parse().unwrap() }
sty -> SeverityType
    = "d" { SeverityType::Num }
    / "s" { SeverityType::String }
tz -> Timezone
    = "s" { Timezone::Utc }
    / "l" { Timezone::Local }
strftime -> String
    = "{" tchar:tchar* "}" { tchar.into_iter().collect() }
tchar -> char
    = "{{" { OPENED_BRACE.chars().next().unwrap() }
    / "}}" { CLOSED_BRACE.chars().next().unwrap() }
    / [^{}] { match_str.chars().next().unwrap() }
name -> Key
    = [0-9]+ { Key::Id(match_str.parse().expect("expect number")) }
    / [a-zA-Z][a-zA-Z0-9]* { Key::Name(match_str.into()) }
"#);

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Id(usize),
    Name(String),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Align {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SeverityType {
    Num,
    String,
}

#[derive(Debug, Clone, PartialEq)]
enum Timezone {
    Utc,
    Local,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimestampType {
    Num,
    Utc(String),
    Local(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Piece of pattern between placeholders.
    Literal(String),
    /// Message placeholder without spec to avoid unnecessary instructions.
    Message,
    /// Message placeholder with spec.
    MessageExt { fill: char, align: Align, width: usize },
    /// Severity placeholder either numeric or string, but without spec.
    Severity { ty: SeverityType },
    /// Severity placeholder either numeric or string with spec.
    SeverityExt { fill: char, align: Align, width: usize, ty: SeverityType },
    ///
    Timestamp { ty: TimestampType },
    ///
    TimestampExt { fill: char, align: Align, width: usize, ty: TimestampType },
    Placeholder(String, Key),
}

pub fn parse(pattern: &str) -> Result<Vec<Token>, ParseError> {
    expression(&pattern)
}

#[cfg(test)]
mod tests {
    use super::{parse, Align, Key, SeverityType, TimestampType, Token};

    #[test]
    fn literal() {
        let tokens = parse("hello").unwrap();

        assert_eq!(vec![Token::Literal("hello".into())], tokens);
    }

    #[test]
    fn message() {
        let tokens = parse("{message}").unwrap();

        assert_eq!(vec![Token::Message], tokens);
    }

    #[test]
    fn message_spec() {
        let tokens = parse("{message:.<10}").unwrap();

        assert_eq!(vec![Token::MessageExt { fill: '.', align: Align::Left, width: 10 }], tokens);
    }

    #[test]
    fn severity() {
        let tokens = parse("{severity}").unwrap();

        assert_eq!(vec![Token::Severity { ty: SeverityType::String }], tokens);
    }

    #[test]
    fn severity_string() {
        let tokens = parse("{severity:s}").unwrap();

        assert_eq!(vec![Token::Severity { ty: SeverityType::String }], tokens);
    }

    #[test]
    fn severity_num() {
        let tokens = parse("{severity:d}").unwrap();

        assert_eq!(vec![Token::Severity { ty: SeverityType::Num }], tokens);
    }

    #[test]
    fn severity_ext() {
        let tokens = parse("{severity:<10}").unwrap();

        assert_eq!(vec![
            Token::SeverityExt {
                fill: ' ',
                align: Align::Left,
                width: 10,
                ty: SeverityType::String
            }
        ], tokens);
    }

    #[test]
    fn severity_ext_with_fill() {
        let tokens = parse("{severity:.^16}").unwrap();

        assert_eq!(vec![
            Token::SeverityExt {
                fill: '.',
                align: Align::Middle,
                width: 16,
                ty: SeverityType::String
            }
        ], tokens);
    }

    #[test]
    fn severity_ext_with_fill_num() {
        let tokens = parse("{severity:.^16d}").unwrap();

        assert_eq!(vec![
            Token::SeverityExt {
                fill: '.',
                align: Align::Middle,
                width: 16,
                ty: SeverityType::Num
            }
        ], tokens);
    }

    #[test]
    fn severity_ext_with_fill_string() {
        let tokens = parse("{severity:.^16s}").unwrap();

        assert_eq!(vec![
            Token::SeverityExt {
                fill: '.',
                align: Align::Middle,
                width: 16,
                ty: SeverityType::String
            }
        ], tokens);
    }

    #[test]
    fn timestamp() {
        let tokens = parse("{timestamp}").unwrap();

        assert_eq!(vec![Token::Timestamp { ty: TimestampType::Utc("%+".into()) }], tokens);
    }

    #[test]
    fn timestamp_num() {
        let tokens = parse("{timestamp:d}").unwrap();

        assert_eq!(vec![Token::Timestamp { ty: TimestampType::Num }], tokens);
    }

    #[test]
    fn timestamp_utc() {
        let tokens = parse("{timestamp:s}").unwrap();

        assert_eq!(vec![Token::Timestamp { ty: TimestampType::Utc("%+".into()) }], tokens);
    }

    #[test]
    fn timestamp_local() {
        let tokens = parse("{timestamp:l}").unwrap();

        assert_eq!(vec![Token::Timestamp { ty: TimestampType::Local("%+".into()) }], tokens);
    }

    #[test]
    fn timestamp_ext_num() {
        let tokens = parse("{timestamp:^20d}").unwrap();

        assert_eq!(vec![
            Token::TimestampExt {
                fill: ' ',
                align: Align::Middle,
                width: 20,
                ty: TimestampType::Num,
            }
        ], tokens);
    }

    #[test]
    fn timestamp_ext_num_with_fill() {
        let tokens = parse("{timestamp:.<d}").unwrap();

        assert_eq!(vec![
            Token::TimestampExt {
                fill: '.',
                align: Align::Left,
                width: 0,
                ty: TimestampType::Num,
            }
        ], tokens);
    }

    #[test]
    fn timestamp_with_pattern_utc() {
        let tokens = parse("{timestamp:{%Y-%m-%d}s}").unwrap();

        assert_eq!(vec![Token::Timestamp { ty: TimestampType::Utc("%Y-%m-%d".into()) }], tokens);
    }

    #[test]
    fn timestamp_with_pattern_local() {
        let tokens = parse("{timestamp:{%Y-%m-%d}l}").unwrap();

        assert_eq!(vec![Token::Timestamp { ty: TimestampType::Local("%Y-%m-%d".into()) }], tokens);
    }

    #[test]
    fn timestamp_with_pattern_utc_and_braces() {
        let tokens = parse("{timestamp:{%Y-%m-%d {{T}} %H:%M:%S.%.6f}s}").unwrap();

        assert_eq!(vec![
            Token::Timestamp {
                ty: TimestampType::Utc("%Y-%m-%d {T} %H:%M:%S.%.6f".into())
            }
        ], tokens);
    }

    #[test]
    fn timestamp_with_pattern_utc_and_braces_limit() {
        let tokens = parse("{timestamp:{{{%Y-%m-%dT%H:%M:%S.%.6f}}}s}").unwrap();

        assert_eq!(vec![
            Token::Timestamp {
                ty: TimestampType::Utc("{%Y-%m-%dT%H:%M:%S.%.6f}".into())
            }
        ], tokens);
    }

    #[test]
    fn timestamp_ext_with_pattern_and_fill_utc() {
        let tokens = parse("{timestamp:{%Y-%m-%d}.<s}").unwrap();

        assert_eq!(vec![
            Token::TimestampExt {
                fill: '.',
                align: Align::Left,
                width: 0,
                ty: TimestampType::Utc("%Y-%m-%d".into()),
            }
        ], tokens);
    }

    #[test]
    fn timestamp_ext_with_pattern_local() {
        let tokens = parse("{timestamp:{%Y-%m-%dT%H:%M:%S.%.6f}>50l}").unwrap();

        assert_eq!(vec![
            Token::TimestampExt {
                fill: ' ',
                align: Align::Right,
                width: 50,
                ty: TimestampType::Local("%Y-%m-%dT%H:%M:%S.%.6f".into()),
            }
        ], tokens);
    }

    #[test]
    fn placeholder() {
        let tokens = parse("{hello}").unwrap();

        let expected = vec![Token::Placeholder("hello".into(), Key::Name("hello".into()))];
        assert_eq!(expected, tokens);
    }
}
