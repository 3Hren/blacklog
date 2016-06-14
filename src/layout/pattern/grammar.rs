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
use super::{Align, Key, SeverityType, Token, OPENED_BRACE, CLOSED_BRACE};

#[pub]
expression -> Vec<Token>
    = (format / text)+
text -> Token
    = "{{" { Token::Literal(OPENED_BRACE.into()) }
    / "}}" { Token::Literal(CLOSED_BRACE.into()) }
    / [^{}]+ { Token::Literal(match_str.into()) }
format -> Token
    = "{" "message" "}" { Token::Message }
    / "{" "message:" align:align? width:width? "}" {
        Token::MessageExt {
            fill: ' ',
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
        }
    }
    / "{" "message:" fill:fill align:align? width:width? "}" {
        Token::MessageExt {
            fill: fill,
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
        }
    }
    / "{" "severity" "}" { Token::Severity { ty: SeverityType::String } }
    / "{" "severity:" "s}" { Token::Severity { ty: SeverityType::String } }
    / "{" "severity:" "d}" { Token::Severity { ty: SeverityType::Num } }
    / "{" "severity:" align:align? width:width? ty:ty? "}" {
        Token::SeverityExt {
            fill: ' ',
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
            ty: ty.unwrap_or(SeverityType::String),
        }
    }
    / "{" "severity:" fill:fill align:align? width:width? ty:ty? "}" {
        Token::SeverityExt {
            fill: fill,
            align: align.unwrap_or(Align::Left),
            width: width.unwrap_or(0),
            ty: ty.unwrap_or(SeverityType::String),
        }
    }
    / "{" "timestamp" "}" { Token::Timestamp("%+".into()) }
    / "{" "timestamp:" align:align? width:width? "d}" {
        Token::TimestampNum(None, align, width)
    }
    / "{" "timestamp:" fill:fill? align:align? width:width? "d}" {
        Token::TimestampNum(fill, align, width)
    }
    / "{" "timestamp:" strftime:strftime ty:[sl]? "}" {
        Token::Timestamp(strftime)
    }
    / "{" key:name "}" { Token::Placeholder(match_str[1..match_str.len() - 1].into(), key) }
fill -> char
    = . { match_str.chars().next().unwrap() }
align -> Align
    = "<" { Align::Left }
    / ">" { Align::Right }
    / "^" { Align::Middle }
width -> usize
    = [0-9]+ { match_str.parse().unwrap() }
ty -> SeverityType
    = "d" { SeverityType::Num }
    / "s" { SeverityType::String }
strftime -> String
    = "{" schar* "}" { match_str[1..match_str.len() - 1].into() }
schar -> char
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
pub enum Token {
    /// Piece of pattern between placeholders.
    Literal(String),
    /// Message placeholder without spec to avoid unnecessary instructions.
    Message,
    /// Message placeholder with spec.
    MessageExt { fill: char, align: Align, width: usize },
    /// Severity placeholder either numeric or string, but without spec.
    Severity { ty: SeverityType },
    ///
    SeverityExt { fill: char, align: Align, width: usize, ty: SeverityType },
    Timestamp(String), // Spec, Pattern, Type[dsl]
    TimestampNum(Option<char>, Option<Align>, Option<usize>),
    Placeholder(String, Key),
}

pub fn parse(pattern: &str) -> Result<Vec<Token>, ParseError> {
    expression(&pattern)
}

#[cfg(test)]
mod tests {
    use super::{parse, Align, Key, SeverityType, Token};

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

        assert_eq!(vec![Token::Timestamp("%+".into())], tokens);
    }

    #[test]
    fn timestamp_num() {
        let tokens = parse("{timestamp:d}").unwrap();

        assert_eq!(vec![Token::TimestampNum(None, None, None)], tokens);
    }

    #[test]
    fn timestamp_num_with_fill() {
        let tokens = parse("{timestamp:.<d}").unwrap();

        assert_eq!(vec![Token::TimestampNum(Some('.'), Some(Align::Left), None)], tokens);
    }

    #[test]
    fn timestamp_ext() {
        let tokens = parse("{timestamp:{%Y-%m-%d}s}").unwrap();

        assert_eq!(vec![Token::Timestamp("%Y-%m-%d".into())], tokens);
    }

    #[test]
    fn placeholder() {
        let tokens = parse("{hello}").unwrap();

        let expected = vec![Token::Placeholder("hello".into(), Key::Name("hello".into()))];
        assert_eq!(expected, tokens);
    }
}
