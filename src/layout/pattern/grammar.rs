use std::io::Write;

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
// [ ] count := parameter | integer
// [ ] parameter := integer '$'

const OPENED_BRACE: &'static str = "{";
const CLOSED_BRACE: &'static str = "}";

peg! grammar(r#"
use super::{Align, Key, Spec, Token, OPENED_BRACE, CLOSED_BRACE};

#[pub]
expression -> Vec<Token>
    = (format / text)+
text -> Token
    = "{{" { Token::Literal(OPENED_BRACE.into()) }
    / "}}" { Token::Literal(CLOSED_BRACE.into()) }
    / [^{}]+ { Token::Literal(match_str.into()) }
format -> Token
    = "{" "message" "}" { Token::Message(None) }
    / "{" "message:" spec:spec? "}" { Token::Message(spec) }
    / "{" key:name "}" { Token::Placeholder(match_str[1..match_str.len() - 1].into(), key) }
spec -> Spec
    = align:align? width:width? { Spec { align: align, width: width } }
align -> Align
    = "<" { Align::Left }
    / ">" { Align::Right }
    / "^" { Align::Middle }
width -> usize
    = [0-9]+ { match_str.parse().unwrap() }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    align: Option<Align>,
    width: Option<usize>,
}

impl Spec {
    pub fn align(&self) -> &Option<Align> {
        &self.align
    }

    pub fn width(&self) -> &Option<usize> {
        &self.width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(String),
    Message(Option<Spec>),
    Placeholder(String, Key),
}

pub fn parse(pattern: &str) -> Result<Vec<Token>, ParseError> {
    expression(&pattern)
}

#[cfg(test)]
mod tests {
    use super::{parse, Align, Key, Spec, Token};

    #[test]
    fn literal_ast() {
        let tokens = parse("hello").unwrap();

        assert_eq!(vec![Token::Literal("hello".into())], tokens);
    }

    #[test]
    fn message_ast() {
        let tokens = parse("{message}").unwrap();

        assert_eq!(vec![Token::Message(None)], tokens);
    }

    #[test]
    fn message_spec_ast() {
        let tokens = parse("{message:<10}").unwrap();

        let spec = Spec {
            align: Some(Align::Left),
            width: Some(10),
        };
        assert_eq!(vec![Token::Message(Some(spec))], tokens);
    }

    #[test]
    fn placeholder_ast() {
        let tokens = parse("{hello}").unwrap();

        let expected = vec![Token::Placeholder("hello".into(), Key::Name("hello".into()))];
        assert_eq!(expected, tokens);
    }
}
