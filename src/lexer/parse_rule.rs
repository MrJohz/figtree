extern crate regex;

use std::io::prelude::*;
use super::{Lexer, LexToken};

pub struct ParseRule {
    open_test: Box<Fn(char) -> bool>,
    construct: Box<Fn(&mut Lexer) -> Option<LexToken>>,
}

impl ParseRule {
    pub fn matches<F>(re: &str, construct: F) -> Self
        where F: Fn(&mut Lexer) -> Option<LexToken> + 'static {

        let re = regex::Regex::new(re).unwrap();
        ParseRule {
            open_test: Box::new(move |character| {
                let mut s = String::new();
                s.push(character);
                re.is_match(&s)
            }),
            construct: Box::new(construct),
        }
    }

    pub fn range<F>(range: )

    pub fn test(&self, c: char) -> bool {
        (self.open_test)(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use super::super::Lexer;

    #[test]
    fn matches_constructor() {
        let rule = ParseRule::matches("a", |lexer| { None });
        assert!(rule.test('a'));
        assert!(!rule.test('v'));

        let rule = ParseRule::matches(r"\d", |lexer| { None });
        assert!(rule.test('4'));
        assert!(!rule.test('v'));
    }
}
