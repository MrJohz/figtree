extern crate regex;

#[derive(Debug, PartialEq, Eq)]
pub struct Position { line: usize, pos: usize, }

enum TokenKind {
    OpenBrace, CloseBrace,
    OpenBracket, CloseBracket,
    Comma, Colon,
    Identifier(String),
    StringLit(String),
    IntegerLit(i64),
    FloatLit(f64),
    BoolLit(bool),
}

pub struct Token {
    pattern: regex::Regex,
}

impl Token {
    fn new(re: &str) -> Result<Self, regex::Error> {
        Ok(Token {
            pattern: try!(regex::Regex::new(&("^".to_string() + re))),
        })
    }

    fn get_match(&self, inp: &str) -> Option<usize> {
        if let Some((start, end)) = self.pattern.find(inp) {
            Some(end)
        } else {
            None
        }
    }
}

pub struct Lexer {
    ignore: Vec<Token>,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            ignore: Vec::new(),
        }
    }

    pub fn ignore(&mut self, ignorable: &str) -> &mut Self {
        if let Ok(ignorable) = Token::new(ignorable) {
            self.ignore.push(ignorable);
        }

        self
    }

    pub fn lex(self) -> LexIter {
        LexIter {
            position: Position {line: 0, pos: 0},
            ignore: self.ignore,
        }
    }
}

pub struct LexIter {
    position: Position,
    ignore: Vec<Token>,
}

#[cfg(test)]
mod tests {
    pub use super::*;

    mod token {
        use super::*;
        extern crate regex;

        #[test]
        fn test_construction() {
            let tok = Token::new("xyz").unwrap();
            assert_eq!(tok.pattern, regex::Regex::new("^xyz").unwrap());
        }

        #[test]
        fn test_matching() {
            let tok = Token::new("xyz").unwrap();
            assert_eq!(tok.get_match("xyzdjsd"), Some(3));
            assert_eq!(tok.get_match("abcdjsd"), None);
        }
    }

    mod lexer {
        use super::*;

        #[test]
        fn test_construction() {
            let mut lexer = Lexer::new();
            lexer.ignore("#*");
            assert_eq!(lexer.ignore.len(), 1);
        }
    }
}
