use super::Token;

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
                  "(?: [^"\n\\] | \\. )*"
                | '(?: [^'\n\\] | \\. )*'
            "#).unwrap(),
            integerlit: Token::new(r#"(?x)
                  0[xX][0-9a-fA-F_]+
                | 0[oO][0-8_]+
                | 0[dD][\d_]+
                | [\d_]+
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

#[cfg(test)]
mod tests {
    use super::*;

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
