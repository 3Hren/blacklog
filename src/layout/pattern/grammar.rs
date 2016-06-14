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
        Token::MessageSpec(Some(' '), align, width)
    }
    / "{" "message:" fill:fill? align:align? width:width? "}" {
        Token::MessageSpec(fill, align, width)
    }
    / "{" "severity" "}" { Token::Severity(None, None, SeverityType::String) }
    / "{" "severity:" align:align? width:width? ty:ty? "}" {
        match ty {
            Some(ty) => Token::Severity(align, width, ty),
            None => Token::Severity(align, width, SeverityType::String),
        }
    }
    / "{" "timestamp" "}" { Token::Timestamp("%+".into()) }
    / "{" "timestamp:" align:align? width:width? "d}" {
        Token::TimestampNum(None, align, width)
    }
    / "{" "timestamp:" fill:fill? align:align? width:width? "d}" {
        Token::TimestampNum(fill, align, width)
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
name -> Key
    = [0-9]+ { Key::Id(match_str.parse().expect("expect number")) }
    / [a-zA-Z][a-zA-Z0-9]* { Key::Name(match_str.into()) }
"#);

// TODO: Format spec.

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
    Literal(String),
    Message,
    MessageSpec(Option<char>, Option<Align>, Option<usize>),
    Severity(Option<Align>, Option<usize>, SeverityType),
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
    fn literal_ast() {
        let tokens = parse("hello").unwrap();

        assert_eq!(vec![Token::Literal("hello".into())], tokens);
    }

    #[test]
    fn message_ast() {
        let tokens = parse("{message}").unwrap();

        assert_eq!(vec![Token::Message], tokens);
    }

    #[test]
    fn message_spec_ast() {
        let tokens = parse("{message:.<10}").unwrap();

        assert_eq!(vec![Token::MessageSpec(Some('.'), Some(Align::Left), Some(10))], tokens);
    }

    #[test]
    fn severity_ast() {
        let tokens = parse("{severity}").unwrap();

        assert_eq!(vec![Token::Severity(None, None, SeverityType::String)], tokens);
    }

    #[test]
    fn severity_with_ty_ast() {
        let tokens = parse("{severity:d}").unwrap();

        assert_eq!(vec![Token::Severity(None, None, SeverityType::Num)], tokens);
    }

    #[test]
    fn timestamp_ast() {
        let tokens = parse("{timestamp}").unwrap();

        assert_eq!(vec![Token::Timestamp("%+".into())], tokens);
    }

    #[test]
    fn timestamp_num_ast() {
        let tokens = parse("{timestamp:d}").unwrap();

        assert_eq!(vec![Token::TimestampNum(None, None, None)], tokens);
    }

    #[test]
    fn timestamp_num_with_fill_ast() {
        let tokens = parse("{timestamp:.<d}").unwrap();

        assert_eq!(vec![Token::TimestampNum(Some('.'), Some(Align::Left), None)], tokens);
    }

    // #[test]
    // fn timestamp_ext_ast() {
    //     let tokens = parse("{timestamp:{%Y-%m-%d}s}").unwrap();
    //
    //     assert_eq!(vec![Token::Timestamp("%Y-%m-%d".into())], tokens);
    // }

    #[test]
    fn placeholder_ast() {
        let tokens = parse("{hello}").unwrap();

        let expected = vec![Token::Placeholder("hello".into(), Key::Name("hello".into()))];
        assert_eq!(expected, tokens);
    }
}
