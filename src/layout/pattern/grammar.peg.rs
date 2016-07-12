use meta::format::Alignment;

use super::{
    FormatSpec,
    SeverityType,
    Timezone,
    Token,
    CLOSED_BRACE,
    OPENED_BRACE
};

#[pub]
expression -> Vec<Token<'input>>
    = (format / text)+
    / { Vec::new() }
text -> Token<'input>
    = "{{" { Token::Piece(OPENED_BRACE) }
    / "}}" { Token::Piece(CLOSED_BRACE) }
    / [^{}]+ { Token::Piece(match_str) }
format -> Token<'input>
    = "{" "message" "}" { Token::Message(None) }
    / "{" "message:" fill:fill? align:align? width:width? precision:precision? "}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: precision,
            width: width.unwrap_or(0),
        };

        Token::Message(Some(spec))
    }
    / "{" "severity" "}"   { Token::Severity(None, SeverityType::String) }
    / "{" "severity:" "s}" { Token::Severity(None, SeverityType::String) }
    / "{" "severity:" "d}" { Token::Severity(None, SeverityType::Num) }
    / "{" "severity:" fill:fill? align:align? width:width? precision:precision? ty:sevty? "}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: precision,
            width: width.unwrap_or(0),
        };

        Token::Severity(Some(spec), ty.unwrap_or(SeverityType::String))
    }
    / "{" "timestamp" "}"   { Token::Timestamp(None, "%+".into(), Timezone::Utc) }
    / "{" "timestamp:" "d}" { Token::TimestampNum(None) }
    / "{" "timestamp:" fill:fill? align:align? width:width? "d}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: None,
            width: width.unwrap_or(0),
        };

        Token::TimestampNum(Some(spec))
    }
    / "{" "timestamp:" pattern:strftime? tz:tz "}" {
        Token::Timestamp(None, pattern.unwrap_or("%+".into()), tz)
    }
    / "{" "timestamp:" pattern:strftime? fill:fill? align:align? width:width? precision:precision? tz:tz "}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: precision,
            width: width.unwrap_or(0),
        };

        Token::Timestamp(Some(spec), pattern.unwrap_or("%+".into()), tz)
    }
    / "{" "line" "}" { Token::Line(None) }
    / "{" "line:" fill:fill? align:align? width:width? "}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: None,
            width: width.unwrap_or(0),
        };

        Token::Line(Some(spec))
    }
    / "{" "module" "}" { Token::Module(None) }
    / "{" "module:" fill:fill? align:align? width:width? precision:precision? "}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: precision,
            width: width.unwrap_or(0),
        };

        Token::Module(Some(spec))
    }
    / "{" "..." "}" { Token::MetaList(None) }
    / "{" name:name "}" { Token::Meta(name, None) }
    / "{" name:name ":" fill:fill? align:align? width:width? precision:precision? "}" {
        let spec = FormatSpec {
            fill: fill.unwrap_or(' '),
            align: align.unwrap_or(Alignment::AlignLeft),
            flags: 0,
            precision: precision,
            width: width.unwrap_or(0),
        };

        Token::Meta(name, Some(spec))
    }
fill -> char
    = . &align { match_str.chars().next().unwrap() }
align -> Alignment
    = "<" { Alignment::AlignLeft }
    / ">" { Alignment::AlignRight }
    / "^" { Alignment::AlignCenter }
width -> usize
    = [0-9]+ { match_str.parse().unwrap() }
precision -> usize
    = "." [0-9]+ { match_str[1..].parse().unwrap() }
sevty -> SeverityType
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
name -> &'input str
    = [a-zA-Z][a-zA-Z0-9]* { match_str }
