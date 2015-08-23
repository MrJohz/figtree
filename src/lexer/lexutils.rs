use super::Token;
use std::char;

pub struct TokenCollection {
    pub open_brace: Token,
    pub close_brace: Token,
    pub open_bracket: Token,
    pub close_bracket: Token,
    pub comma: Token,
    pub colon: Token,
    pub identifier: Token,
    pub stringlit: Token,
    pub integerlit: Token,
    pub floatlit: Token,
}

impl TokenCollection {
    pub fn new() -> Self {
        TokenCollection {
            open_brace: Token::new(r"\{").unwrap(),
            close_brace: Token::new(r"\}").unwrap(),
            open_bracket: Token::new(r"\[").unwrap(),
            close_bracket: Token::new(r"\]").unwrap(),
            comma: Token::new(r",").unwrap(),
            colon: Token::new(r":").unwrap(),
            identifier: Token::new(r#"(?x)
                [
                    \p{L} # letters
                    \p{M} # combining marks
                    \p{Pc} # connector punctuation (e.g. _)
                ]
                \w* # all word characters
            "#).unwrap(),
            stringlit: Token::new(r#"(?x)
                  "(?: [^"\n\\] | \\n | \\r | \\t | \\\\ | \\b
                    | \\f | \\/ | \\" | \\' | \\u[0-9a-fA-F]{4} )*"
                | '(?: [^'\n\\] | \\n | \\r | \\t | \\\\ | \\b
                    | \\f | \\/ | \\" | \\' | \\u[0-9a-fA-F]{4} )*'
            "#).unwrap(),
            integerlit: Token::new(r#"(?x)
                  0[xX][0-9a-fA-F_]+
                | 0[oO][0-8_]+
                | 0[bB][10_]+
                | 0[dD][\d_]+
                | [+-]?[\d_]+
            "#).unwrap(),
            floatlit: Token::new(r#"(?x)
                [-+]? # optional sign
                (?:
                      \d+\.\d*  # numbers <dot> <optional numbers>
                    | \.\d+     # <dot> numbers
                )
                (?:
                    [eE]  # begin exponent
                    [+-]? # sign
                    \d+   # exponent
                )? # optional exponent part
            "#).unwrap(),
        }
    }
}

pub fn parse_string(inp: String) -> String {
    let mut escaped = false;
    let mut unicode = false;
    let mut uindex = 0;
    let mut uvalue = 0;
    let quote_char = inp.chars().next().expect("parse_string is parsing an empty string");
    inp.chars().filter_map(|character| {
        if escaped {
            escaped = false;
            match character {
                'n' => Some('\n'),
                'r' => Some('\r'),
                't' => Some('\t'),
                'b' => Some('\x08'),
                'f' => Some('\x0c'),
                'u' => { unicode = true; uvalue = 0; uindex = 0; None }
                 _  => Some(character),
            }
        } else if unicode {
            // code adapted from https://github.com/rust-lang/rustc-serialize/blob/master/src/json.rs#L1608-L1624
            // original licensed under MIT/Apache-2.0
            uvalue = match character {
                ctr @ '0'...'9' => uvalue * 16 + ((ctr as u16) - ('0' as u16)),
                ctr @ 'a'...'f' => uvalue * 16 + (10 + (ctr as u16) - ('a' as u16)),
                ctr @ 'A'...'F' => uvalue * 16 + (10 + (ctr as u16) - ('A' as u16)),
                _ => return None,
            };
            if uindex < 3 {
                uindex += 1;
                None
            } else {
                unicode = false;
                char::from_u32(uvalue as u32)
            }
        } else if character == quote_char {
            None
        } else if character == '\\' {
            escaped = true;
            None
        } else {
            Some(character)
        }
    }).collect()
}

pub fn parse_integer(inp: String) -> i64 {
    if inp.starts_with("0d") || inp.starts_with("0D") {
        i64::from_str_radix(&inp.chars().skip(2).collect::<String>(), 10).unwrap()
    } else if inp.starts_with("0x") || inp.starts_with("0X") {
        i64::from_str_radix(&inp.chars().skip(2).collect::<String>(), 16).unwrap()
    } else if inp.starts_with("0o") || inp.starts_with("0O") {
        i64::from_str_radix(&inp.chars().skip(2).collect::<String>(), 8).unwrap()
    } else if inp.starts_with("0b") || inp.starts_with("0B") {
        i64::from_str_radix(&inp.chars().skip(2).collect::<String>(), 2).unwrap()
    } else if inp.starts_with("+") {
        i64::from_str_radix(&inp.chars().skip(1).collect::<String>(), 10).unwrap()
    } else {
        i64::from_str_radix(&inp, 10).unwrap()
    }
}

pub fn parse_float(inp: String) -> f64 {
    if inp.starts_with("+") {
        inp.chars().skip(1).collect::<String>().parse().unwrap()
    } else {
       inp.parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string("'hello'".to_string()), "hello");
        assert_eq!(parse_string(r#""hello""#.to_string()), "hello");
        assert_eq!(parse_string("'he\\nllo'".to_string()), "he\nllo");
        assert_eq!(parse_string(r#"'he"llo'"#.to_string()), "he\"llo");
        assert_eq!(parse_string(r"'\u0058'".to_string()), "\u{0058}");
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_integer("0d5".to_string()), 5);
        assert_eq!(parse_integer("0D5474".to_string()), 5474);
        assert_eq!(parse_integer("0x4f3".to_string()), 0x4f3);
        assert_eq!(parse_integer("0X7Df31".to_string()), 0x7df31);
        assert_eq!(parse_integer("0o443".to_string()), 3 + (8*4) + (64*4));
        assert_eq!(parse_integer("0O70131".to_string()), 28761);
        assert_eq!(parse_integer("0b01101".to_string()), 0b01101);
        assert_eq!(parse_integer("0B10010".to_string()), 0b10010);
        assert_eq!(parse_integer("32353".to_string()), 32353);
        assert_eq!(parse_integer("-1234".to_string()), -1234);
        assert_eq!(parse_integer("+4321".to_string()), 4321);
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float("+1.2".to_string()), 1.2);
        assert_eq!(parse_float("-1.2".to_string()), -1.2);
        assert_eq!(parse_float("3.4".to_string()), 3.4);
        assert_eq!(parse_float("3.".to_string()), 3.0);
        assert_eq!(parse_float(".2".to_string()), 0.2);
        assert_eq!(parse_float("1.2e2".to_string()), 1.2E+2);
        assert_eq!(parse_float("1.2e+2".to_string()), 1.2E+2);
        assert_eq!(parse_float("1.2e-2".to_string()), 1.2E-2);
        assert_eq!(parse_float("1.2E2".to_string()), 1.2E+2);
        assert_eq!(parse_float("1.2E+2".to_string()), 1.2E+2);
        assert_eq!(parse_float("1.2E-2".to_string()), 1.2E-2);
    }

    #[test]
    fn literals_match() {
        let tk = TokenCollection::new();
        assert_eq!(tk.open_brace.get_match("{"), Some(1));
        assert_eq!(tk.open_brace.get_match("}"), None);
        assert_eq!(tk.close_brace.get_match("}"), Some(1));
        assert_eq!(tk.close_brace.get_match("d"), None);
        assert_eq!(tk.open_bracket.get_match("["), Some(1));
        assert_eq!(tk.open_bracket.get_match(" "), None);
        assert_eq!(tk.close_bracket.get_match("]"), Some(1));
        assert_eq!(tk.close_bracket.get_match("["), None);
        assert_eq!(tk.comma.get_match(","), Some(1));
        assert_eq!(tk.comma.get_match(""), None);
        assert_eq!(tk.colon.get_match(":"), Some(1));
        assert_eq!(tk.colon.get_match(";"), None);
    }

    #[test]
    fn identifier_matches() {
        let tk = TokenCollection::new();
        assert_eq!(tk.identifier.get_match("hello"), Some(5));
        assert_eq!(tk.identifier.get_match("HELLO"), Some(5));
        assert_eq!(tk.identifier.get_match("45fefe"), None);
        assert_eq!(tk.identifier.get_match("fefe45"), Some(6));
        assert_eq!(tk.identifier.get_match("true"), Some(4));
        assert_eq!(tk.identifier.get_match("$"), None);
    }

    #[test]
    fn string_matches() {
        let tk = TokenCollection::new();
        assert_eq!(tk.stringlit.get_match(""), None);
        assert_eq!(tk.stringlit.get_match("hello"), None);
        assert_eq!(tk.stringlit.get_match("'hello'"), Some(7));
        assert_eq!(tk.stringlit.get_match("{}[],: 'string'"), None);
        assert_eq!(tk.stringlit.get_match(r#""hello""#), Some(7));
        assert_eq!(tk.stringlit.get_match(r"'hello\n'"), Some(9));
        assert_eq!(tk.stringlit.get_match(r"'hello\''"), Some(9));
        assert_eq!(tk.stringlit.get_match(r"'
            no newline in strings'"), None);
    }

    #[test]
    fn integer_matches() {
        let tk = TokenCollection::new();
        assert_eq!(tk.integerlit.get_match("345"), Some(3));
        assert_eq!(tk.integerlit.get_match("3_4_5"), Some(5));
        assert_eq!(tk.integerlit.get_match("0d3_4_5"), Some(7));
        assert_eq!(tk.integerlit.get_match("0x45"), Some(4));
        assert_eq!(tk.integerlit.get_match("0Xff"), Some(4));
        assert_eq!(tk.integerlit.get_match("0xf_f"), Some(5));
        assert_eq!(tk.integerlit.get_match("0xgg"), Some(1));
        assert_eq!(tk.integerlit.get_match("0o44"), Some(4));
        assert_eq!(tk.integerlit.get_match("0O44"), Some(4));
        assert_eq!(tk.integerlit.get_match("0o99"), Some(1));
    }

    #[test]
    fn float_matches() {
        let tk = TokenCollection::new();
        assert_eq!(tk.floatlit.get_match("300."), Some(4));
        assert_eq!(tk.floatlit.get_match("30.0"), Some(4));
        assert_eq!(tk.floatlit.get_match(".300"), Some(4));
        assert_eq!(tk.floatlit.get_match("300"), None);
        assert_eq!(tk.floatlit.get_match("3.e5"), Some(4));
        assert_eq!(tk.floatlit.get_match("3.e-5"), Some(5));
        assert_eq!(tk.floatlit.get_match("3.E-5"), Some(5));
    }
}
