use std::io::prelude::*;
use std::io;
use std::char::from_u32;
use std::str::FromStr;

use utils::{CharReader, ident_head, ident_body};
use position::MutablePosition;

type LexResult = Result<LexToken, LexError>;

/// A enum representing different kinds of lexed event
#[derive(Debug, PartialEq, Clone)]
pub enum LexToken {
    OpenBrace, CloseBrace,
    OpenBracket, CloseBracket,
    Comma, Colon, Bang,
    Identifier(String),
    StringLit(String),
    IntegerLit(i64),
    FloatLit(f64),
}

/// An enum representing different kinds of lexing errors
///
/// May be referenced in a `ParseError` if the parsing failed due to a lexical error
#[derive(Debug, PartialEq)]
pub enum LexError {
    UnclosedCommentError,
    UnclosedStringError,
    UnclosedIdentError,
    NewlineInIdentifier,
    InvalidEscape(char),
    InvalidUnicodeEscape(u32),
    FloatParseError(<f64 as FromStr>::Err),
    IntegerParseError(<i64 as FromStr>::Err),
    UnrecognisedCharError(char),
}

pub struct Lexer {
    pub token_start: MutablePosition,
    pub position: MutablePosition,
    input: CharReader<io::BufReader<Box<Read>>>,
    stored_next: Vec<char>,
    errored: bool,
    peeked_next: Option<LexResult>,
}

impl Lexer {
    pub fn lex<R: Read + 'static>(reader: R) -> Self {
        Lexer {
            input: CharReader::new(io::BufReader::new(Box::new(reader))),
            token_start: MutablePosition::new(),
            position: MutablePosition::new(),
            stored_next: Vec::new(),
            errored: false,
            peeked_next: None,
        }
    }

    pub fn peek(&mut self) -> Option<&LexResult> {
        if self.peeked_next.is_none() {
            self.peeked_next = self.next();
        }

        self.peeked_next.as_ref()
    }

    fn err(&mut self, err: LexError) -> Option<LexResult> {
        self.errored = true;
        Some(Err(err))
    }

    fn pop_next(&mut self) -> Option<char> {
        if let Some(next) =
            if let Some(next) = self.stored_next.pop() { Some(next) }
            else { self.input.next() } {

            if next == '\n' {
                self.position.new_line();
            } else {
                self.position.push(1);
            }

            Some(next)
        } else {
            None
        }
    }

    fn ret_next(&mut self, returned: char) {
        self.position.unpush(1);
        self.stored_next.push(returned);
    }

    fn parse_ident(&mut self) -> Option<LexResult> {
        if let Some(next_char) = self.pop_next() {
            let mut ident = String::new();
            if ident_head(next_char) {
                ident.push(next_char);
            } else {
                self.ret_next(next_char);
                return None;
            }

            while let Some(next_char) = self.pop_next() {
                if ident_body(next_char) {
                    ident.push(next_char);
                } else if next_char == '_' {
                    ident.push(next_char);
                } else {
                    self.ret_next(next_char);
                    break;
                }
            }

            Some(Ok(LexToken::Identifier(ident)))
        } else {
            None
        }
    }

    fn parse_int(&mut self, base: u32) -> Option<LexResult> {
        let mut buffer = String::new();

        while let Some(next_char) = self.pop_next() {
            if next_char.is_digit(base) {
                buffer.push(next_char);
            } else if next_char == '_' {
                continue; // accepted but ignored
            } else {
                self.ret_next(next_char);
                break;
            }
        }

        Some(Ok(LexToken::IntegerLit(i64::from_str_radix(&buffer, base).unwrap())))
    }

    fn parse_exponent(&mut self) -> String {
        let mut exponent = String::from("e");
        if let Some(next_char) = self.pop_next() {
            if next_char == '+' || next_char == '-' {
                exponent.push(next_char);
            } else {
                self.ret_next(next_char);
            }
        }

        while let Some(next_char) = self.pop_next() {
            if next_char.is_digit(10) {
                exponent.push(next_char)
            } else {
                self.ret_next(next_char);
                break;
            }
        }

        exponent
    }

    fn parse_float_int(&mut self) -> Option<LexResult> {
        let mut sign = '+';
        let mut is_float = false;
        let mut buffer = String::new();
        let mut exponent = String::new();

        if let Some(next_char) = self.pop_next() {
            if next_char == '+' || next_char == '-' {
                sign = next_char;
            } else {
                self.ret_next(next_char);
            }
        }

        while let Some(next_char) = self.pop_next() {
            if next_char == '.' {
                if is_float {
                    // already a float - can't have two periods!
                    self.ret_next(next_char);
                    break;
                } else {
                    is_float = true;
                    buffer.push(next_char);
                }
            } else if next_char.is_digit(10) {
                buffer.push(next_char);
            } else if next_char == '_' {
                continue;  // accepted, but ignored
            } else if next_char == 'e' || next_char == 'E' {
                exponent = self.parse_exponent();
                is_float = true;
                break;
            } else {
                self.ret_next(next_char);
                break;
            }
        }

        if is_float {
            let str_float = buffer + &exponent;
            Some(str_float.parse::<f64>()
                .map(move |flt| {
                    if sign == '+' {
                        LexToken::FloatLit(flt)
                    } else {
                        LexToken::FloatLit(-flt)
                    }
                })
                .map_err(LexError::FloatParseError))
        } else {
            Some(buffer.parse::<i64>()
                .map(move |intgr| {
                    if sign == '+' {
                        LexToken::IntegerLit(intgr)
                    } else {
                        LexToken::IntegerLit(-intgr)
                    }
                })
                .map_err(LexError::IntegerParseError))
        }
    }

    fn parse_numeric(&mut self) -> Option<LexResult> {
        if let Some(next_char) = self.pop_next() {
            if let Some(after) = self.pop_next() {
                if next_char == '0' && ['d', 'D'].contains(&after) {
                    self.parse_int(10)
                } else if next_char == '0' && ['x', 'X'].contains(&after)  {
                    self.parse_int(16)
                } else if next_char == '0' && ['o', 'O'].contains(&after) {
                    self.parse_int(8)
                } else if next_char == '0' && ['b', 'B'].contains(&after) {
                    self.parse_int(2)
                } else {
                    self.ret_next(after);      // put back the two characters we popped
                    self.ret_next(next_char);  // off, but in the right order
                    self.parse_float_int()
                }
            } else {
                Some(Ok(LexToken::IntegerLit(next_char.to_digit(10).unwrap() as i64)))
            }
        } else {
            None
        }
    }

    fn parse_unicode(&mut self) -> Result<char, LexError> {
        let mut uvalue = 0;
        for _ in 0..4 {
            if let Some(ch) = self.pop_next() {
                uvalue = match ch {
                    '0'...'9' =>
                        uvalue * 16 + ((ch as u16) - ('0' as u16)),
                    'a'...'f' =>
                        uvalue * 16 + (10 + (ch as u16) - ('a' as u16)),
                    'A'...'F' =>
                        uvalue * 16 + (10 + (ch as u16) - ('A' as u16)),
                    _ => {
                        return Err(LexError::InvalidUnicodeEscape(uvalue as u32));
                    },
                };
            } else {
                return Err(LexError::UnclosedStringError);
            }
        }

        if let Some(next_char) = from_u32(uvalue as u32) {
            Ok(next_char)
        } else {
            Err(LexError::InvalidUnicodeEscape(uvalue as u32))
        }
    }

    fn parse_string(&mut self) -> Option<LexResult> {
        let mut buffer = String::new();
        let mut quote_closed = false;
        let quote_char = match self.pop_next() {
            Some('\'') => '\'',
            Some('\"') => '\"',
            Some(ch) => unreachable!("{:?} should not be a quote char", ch),
            None => { return None; },
        };

        while let Some(next_char) = self.pop_next() {
            if next_char == '\\' {
                // escape next character
                match self.pop_next() {
                    Some('/') => { buffer.push('/'); },
                    Some('n') => { buffer.push('\n'); },
                    Some('r') => { buffer.push('\r'); },
                    Some('t') => { buffer.push('\t'); },
                    Some('\\') => { buffer.push('\\'); },
                    Some('\"') => { buffer.push('\"'); },
                    Some('\'') => { buffer.push('\''); },
                    Some('b') => { buffer.push('\x08'); },
                    Some('f') => { buffer.push('\x0c'); },
                    Some('u') => {
                        match self.parse_unicode() {
                            Ok(next_char) => { buffer.push(next_char); },
                            Err(err) => { return self.err(err); }
                        }
                    },
                    Some(c) => {
                        return self.err(LexError::InvalidEscape(c));
                    },
                    None => {
                        return self.err(LexError::UnclosedStringError);
                    },
                }
            } else if next_char == quote_char {
                quote_closed = true;
                break;
            } else {
                buffer.push(next_char);
            }
        }

        if quote_closed {
            Some(Ok(LexToken::StringLit(buffer)))
        } else {
            self.err(LexError::UnclosedStringError)
        }
    }

    fn parse_raw_string(&mut self) -> Option<LexResult> {
        let mut buffer = String::new();
        let mut quote_closed = false;
        let mut quote_length = 1;
        assert!(self.pop_next() == Some('r')); // otherwise something wrong has happened
        let (quote_char, closed_char) = match self.pop_next() {
            Some('/') => ('/', '/'),
            Some('|') => ('|', '|'),
            Some('#') => ('#', '#'),
            Some('$') => ('$', '$'),
            Some('%') => ('%', '%'),
            Some('(') => ('(', ')'), // N.B. special
            Some('"') => ('"', '"'),
            Some('\'') => ('\'', '\''),
            Some(ch) => unreachable!("{:?} should not be a quote char", ch),
            None => { return None; },
        };

        while let Some(next_char) = self.pop_next() {
            if next_char == quote_char {
                quote_length += 1;
            } else {
                self.ret_next(next_char);
                break;
            }
        }

        while let Some(next_char) = self.pop_next() {
            if next_char == closed_char {
                let mut close_quote_length = 1;
                while let Some(next_char) = self.pop_next() {
                    if next_char == closed_char {
                        close_quote_length += 1;
                        if close_quote_length == quote_length {
                            quote_closed = true;
                            break;
                        }
                    } else {
                        for _ in 0..(close_quote_length) {
                            buffer.push(closed_char);
                        }
                        buffer.push(next_char);
                        break;
                    }
                }

                if quote_closed {
                    break;
                }
            } else {
                buffer.push(next_char);
            }
        }

        Some(Ok(LexToken::StringLit(buffer)))
    }

    fn parse_ident_escaped(&mut self) -> Option<LexResult> {
        let mut buffer = String::new();
        let mut quote_closed = false;
        match self.pop_next() {
            Some('`') => '`',
            Some(ch) => unreachable!("{:?} should not be a quote char", ch),
            None => { return None; },
        };

        while let Some(next_char) = self.pop_next() {
            if next_char == '\\' {
                // escape next character
                match self.pop_next() {
                    Some('`') => { buffer.push('`'); },
                    Some('/') => { buffer.push('/'); },
                    Some('n') => { buffer.push('\n'); },
                    Some('r') => { buffer.push('\r'); },
                    Some('t') => { buffer.push('\t'); },
                    Some('\\') => { buffer.push('\\'); },
                    Some('b') => { buffer.push('\x08'); },
                    Some('f') => { buffer.push('\x0c'); },
                    Some('u') => {
                        match self.parse_unicode() {
                            Ok(next_char) => { buffer.push(next_char); },
                            Err(err) => { return self.err(err); }
                        }
                    },
                    Some(c) => {
                        return self.err(LexError::InvalidEscape(c));
                    },
                    None => {
                        return self.err(LexError::UnclosedIdentError);
                    },
                }
            } else if next_char == '\n' {
                return self.err(LexError::NewlineInIdentifier)
            } else if next_char == '`' {
                quote_closed = true;
                break;
            } else {
                buffer.push(next_char);
            }
        }

        if quote_closed {
            Some(Ok(LexToken::Identifier(buffer)))
        } else {
            self.err(LexError::UnclosedIdentError)
        }
    }

    fn remove_line_comment(&mut self) -> Option<LexError> {
        while let Some(ch) = self.pop_next() {
            if ch == '\r' || ch == '\n' {
                break;
            }
        }

        None
    }
    fn remove_multiline_comment(&mut self) -> Option<LexError> {
        let mut comment_level = 1;
        loop {
            if let Some(ch) = self.pop_next() {
                if ch == '/' {
                    if let Some(ch) = self.pop_next() {
                        if ch == '*' {
                            comment_level += 1;
                        } else {
                            self.ret_next(ch);
                        }
                    }
                } else if ch == '*' {
                    if let Some(ch) = self.pop_next() {
                        if ch == '/' {
                            comment_level -= 1;
                        } else {
                            self.ret_next(ch);
                        }
                    }
                }

                if comment_level == 0 {
                    break;
                }
            } else {
                return Some(LexError::UnclosedCommentError);
            }
        }

        None
    }
}

impl Iterator for Lexer {
    type Item = Result<LexToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked_next.is_some() {
            return self.peeked_next.take();
        }

        // remove comments & whitespace (ignorables)
        // loop continuously until told to break
        loop {
            // take first character, test if it's either whitespace or '/'
            if let Some(ch) = self.pop_next() {
                if ch.is_whitespace() {
                    continue;
                } else if ch == '/' {
                    match self.pop_next() {
                        Some('/') => {
                            // single line comment ("// hello")
                            // remove, and continue cycle to find next ignorable
                            if let Some(err) = self.remove_line_comment() {
                                return self.err(err);
                            }
                            continue;
                        },
                        Some('*') => {
                            // multiline comment ("/* hello */")
                            // remove and continue cycle
                            if let Some(err) = self.remove_multiline_comment() {
                                return self.err(err);
                            }
                            continue;
                        },
                        Some(ch) => {
                            // neither single nor multiline comment
                            // nor is it whitespace (failed that check earlier)
                            // return both popped characters and break out of loop
                            self.ret_next(ch);
                            self.ret_next('/');
                            break;
                        },
                        None => {
                            // Best place to deal with unexpected EOFs is in the main
                            // lexer body, so break here and let it be dealt with
                            // there.
                            break;
                        }
                    }
                } else {
                    // not a whitespace, not a comment
                    // return popped character and break out of loop
                    self.ret_next(ch);
                    break;
                }
            } else {
                // None -> Pass to main lexer body, that knows how to deal with it best
                break;
            }
        }

        self.token_start = self.position.clone();

        if let Some(next_char) = self.pop_next() {
            if next_char == 'r' {
                if let Some(after) = self.pop_next() {
                    match after {
                        '/' | '|' | '#' | '"' | '\'' | '$' | '%' | '(' => {
                            self.ret_next(after);
                            self.ret_next(next_char);
                            return self.parse_raw_string();
                        },
                        _ => {
                            self.ret_next(after);
                            self.ret_next(next_char);
                        }
                    }
                } else {
                    self.ret_next(next_char);
                }
            }
            if next_char == '{' {
                return Some(Ok(LexToken::OpenBrace));
            }
            if next_char == '}' {
                return Some(Ok(LexToken::CloseBrace));
            }
            if next_char == '[' {
                return Some(Ok(LexToken::OpenBracket));
            }
            if next_char == ']' {
                return Some(Ok(LexToken::CloseBracket));
            }
            if next_char == ',' {
                return Some(Ok(LexToken::Comma));
            }
            if next_char == '!' {
                return Some(Ok(LexToken::Bang));
            }
            if next_char == ':' {
                return Some(Ok(LexToken::Colon));
            }
            if next_char == '`' {
                self.ret_next(next_char);
                return self.parse_ident_escaped();
            }
            if ident_head(next_char) {
                self.ret_next(next_char);
                return self.parse_ident();
            }
            if next_char.is_digit(10) || ['+', '-', '.'].contains(&next_char) {
                self.ret_next(next_char);
                return self.parse_numeric();
            }
            if next_char == '\"' || next_char == '\'' {
                self.ret_next(next_char);
                return self.parse_string();
            }

            Some(Err(LexError::UnrecognisedCharError(next_char)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::position::MutablePosition;
    use std::io::Cursor;

    #[test]
    fn iterator() {
        let cursor = Cursor::new(
            "ident { } [ ! ] : , 34 3.5 'str' true r/raw string/"
        .as_bytes());
        let mut lexer = Lexer::lex(cursor);
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::Identifier("ident".to_string()));
        assert_eq!(lexer.token_start, MutablePosition::at(0, 0));
        assert_eq!(lexer.position, MutablePosition::at(0, 5));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::OpenBrace);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 6));
        assert_eq!(lexer.position, MutablePosition::at(0, 7));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::CloseBrace);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 8));
        assert_eq!(lexer.position, MutablePosition::at(0, 9));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::OpenBracket);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 10));
        assert_eq!(lexer.position, MutablePosition::at(0, 11));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::Bang);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 12));
        assert_eq!(lexer.position, MutablePosition::at(0, 13));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::CloseBracket);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 14));
        assert_eq!(lexer.position, MutablePosition::at(0, 15));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::Colon);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 16));
        assert_eq!(lexer.position, MutablePosition::at(0, 17));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::Comma);
        assert_eq!(lexer.token_start, MutablePosition::at(0, 18));
        assert_eq!(lexer.position, MutablePosition::at(0, 19));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::IntegerLit(34));
        assert_eq!(lexer.token_start, MutablePosition::at(0, 20));
        assert_eq!(lexer.position, MutablePosition::at(0, 22));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::FloatLit(3.5));
        assert_eq!(lexer.token_start, MutablePosition::at(0, 23));
        assert_eq!(lexer.position, MutablePosition::at(0, 26));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::StringLit("str".to_string()));
        assert_eq!(lexer.token_start, MutablePosition::at(0, 27));
        assert_eq!(lexer.position, MutablePosition::at(0, 32));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::Identifier("true".to_string()));
        assert_eq!(lexer.token_start, MutablePosition::at(0, 33));
        assert_eq!(lexer.position, MutablePosition::at(0, 37));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::StringLit("raw string".to_string()));
        assert_eq!(lexer.token_start, MutablePosition::at(0, 38));
        assert_eq!(lexer.position, MutablePosition::at(0, 51));
        assert!(lexer.next().is_none());
    }

    #[test]
    fn ignore_comments() {
        let mut lexer = Lexer::lex(Cursor::new("
            1
            // hello
            2".as_bytes()));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(1));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(2));
        assert!(lexer.next().is_none());

        let mut lexer = Lexer::lex(Cursor::new("
            1
            // hello".as_bytes()));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(1));
        assert!(lexer.next().is_none());

        let mut lexer = Lexer::lex(Cursor::new("
            1 /*
            hello
            */ 2".as_bytes()));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(1));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(2));
        assert!(lexer.next().is_none());

        // nested
        let mut lexer = Lexer::lex(Cursor::new("
            1 /* /*
            hello
            */ */ 2".as_bytes()));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(1));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(2));
        assert!(lexer.next().is_none());

        // error, unclosed
        let mut lexer = Lexer::lex(Cursor::new("1 /*".as_bytes()));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::IntegerLit(1));
        assert_eq!(lexer.next().unwrap().unwrap_err(), LexError::UnclosedCommentError);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn parse_unrecognised_char() {
        let mut lexer = Lexer::lex(Cursor::new("&".as_bytes()));
        match lexer.next() {
            None => panic!("Should return some"),
            Some(Ok(tok)) => panic!(format!("Should return err, returned Ok({:?})", tok)),
            Some(Err(LexError::UnrecognisedCharError(c))) => assert_eq!(c, '&'),
            Some(Err(err)) => panic!(format!("Should return char error, returned {:?}", err)),
        }
    }

    #[test]
    fn parse_ident() {
        let mut lexer = Lexer::lex(Cursor::new("ThisIsAnIdent".as_bytes()));
        assert_eq!(lexer.parse_ident().unwrap().unwrap(),
            LexToken::Identifier("ThisIsAnIdent".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("this_is_an_ident".as_bytes()));
        assert_eq!(lexer.parse_ident().unwrap().unwrap(),
            LexToken::Identifier("this_is_an_ident".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("th15_1s_an_1d3n7".as_bytes()));
        assert_eq!(lexer.parse_ident().unwrap().unwrap(),
            LexToken::Identifier("th15_1s_an_1d3n7".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("7h15_1s_an_1d3n7".as_bytes()));
        assert!(lexer.parse_ident().is_none());

        let mut lexer = Lexer::lex(Cursor::new("th15_1s_an_1d3n7".as_bytes()));
        assert_eq!(lexer.parse_ident().unwrap().unwrap(),
            LexToken::Identifier("th15_1s_an_1d3n7".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("🐶".as_bytes())); // heart
        assert_eq!(lexer.parse_ident().unwrap().unwrap(),
            LexToken::Identifier("🐶".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("`ident`".as_bytes()));
        assert_eq!(lexer.parse_ident_escaped().unwrap().unwrap(),
            LexToken::Identifier("ident".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("`id\\u0041ent`".as_bytes()));
        assert_eq!(lexer.parse_ident_escaped().unwrap().unwrap(),
            LexToken::Identifier("idAent".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("`i\\ndent`".as_bytes()));
        assert_eq!(lexer.parse_ident_escaped().unwrap().unwrap(),
            LexToken::Identifier("i\ndent".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("`i\ndent`".as_bytes()));
        assert_eq!(lexer.parse_ident_escaped().unwrap().unwrap_err(),
            LexError::NewlineInIdentifier);

        let mut lexer = Lexer::lex(Cursor::new("`stri\\\\ng`".as_bytes()));
        assert_eq!(lexer.parse_ident_escaped().unwrap().unwrap(),
            LexToken::Identifier("stri\\ng".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("`string".as_bytes()));
        match lexer.parse_ident_escaped().unwrap() {
            Ok(_) => panic!("should raise error"),
            Err(LexError::UnclosedIdentError) => assert!(true),
            Err(_) => panic!("wrong error raised"),
        }
    }

    #[test]
    fn parse_numeric() {
        let mut lexer = Lexer::lex(Cursor::new("12345".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(12345));

        let mut lexer = Lexer::lex(Cursor::new("0".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(0));

        let mut lexer = Lexer::lex(Cursor::new("0d10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(10));

        let mut lexer = Lexer::lex(Cursor::new("0D10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(10));

        let mut lexer = Lexer::lex(Cursor::new("0b10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(2));

        let mut lexer = Lexer::lex(Cursor::new("0B10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(2));

        let mut lexer = Lexer::lex(Cursor::new("0x10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(16));

        let mut lexer = Lexer::lex(Cursor::new("0X10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(16));

        let mut lexer = Lexer::lex(Cursor::new("0o10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(8));

        let mut lexer = Lexer::lex(Cursor::new("0O10".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(8));

        let mut lexer = Lexer::lex(Cursor::new("0d10e5".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(10));

        let mut lexer = Lexer::lex(Cursor::new("10e5".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::FloatLit(10e5));

        let mut lexer = Lexer::lex(Cursor::new("10.5".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::FloatLit(10.5));

        let mut lexer = Lexer::lex(Cursor::new("105".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(105));

        let mut lexer = Lexer::lex(Cursor::new("1_0__5___".as_bytes()));
        assert_eq!(lexer.parse_numeric().unwrap().unwrap(),
            LexToken::IntegerLit(105));
    }

    #[test]
    fn parse_string() {
        let mut lexer = Lexer::lex(Cursor::new("'string'".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("string".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("\"string\"".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("string".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("\"s\\ntring\"".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("s\ntring".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("'s\ntring'".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("s\ntring".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("'st\\u0041ring'".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("stAring".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("'stri\\\\ng'".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("stri\\ng".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("'str\\'ing'".as_bytes()));
        assert_eq!(lexer.parse_string().unwrap().unwrap(),
            LexToken::StringLit("str'ing".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("'string".as_bytes()));
        match lexer.parse_string().unwrap() {
            Ok(_) => panic!("should raise error"),
            Err(LexError::UnclosedStringError) => assert!(true),
            Err(_) => panic!("wrong error raised"),
        }
    }

    #[test]
    fn parse_raw_string() {
        let mut lexer = Lexer::lex(Cursor::new("r/hello/".as_bytes()));
        assert_eq!(lexer.parse_raw_string().unwrap().unwrap(),
            LexToken::StringLit("hello".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("r////hello////".as_bytes()));
        assert_eq!(lexer.parse_raw_string().unwrap().unwrap(),
            LexToken::StringLit("hello".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("r////hel///lo////".as_bytes()));
        assert_eq!(lexer.parse_raw_string().unwrap().unwrap(),
            LexToken::StringLit("hel///lo".to_string()));

        let mut lexer = Lexer::lex(Cursor::new("r((/hel/(((()//lo))".as_bytes()));
        assert_eq!(lexer.parse_raw_string().unwrap().unwrap(),
            LexToken::StringLit("/hel/(((()//lo".to_string()));
    }
}
