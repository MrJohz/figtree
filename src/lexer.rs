use std::io::prelude::*;
use std::io;
use std::char::from_u32;
use std::str::FromStr;

use utils::CharReader;
use position::Position;

type LexReader<R: Read> = CharReader<io::BufReader<R>>;
type LexResult = Result<LexToken, LexError>;

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

#[derive(Debug)]
pub enum LexError {
    UnclosedStringError,
    InvalidEscape(char),
    InvalidUnicodeEscape(u32),
    IOError(io::Error),
    FloatParseError(<f64 as FromStr>::Err),
    IntegerParseError(<i64 as FromStr>::Err),
    UnrecognisedCharError(char),
}

pub struct Lexer<R: Read> {
    input: LexReader<R>,
    pub position: Position,
    stored_next: Vec<char>,
    errored: bool,
    peeked_next: Option<LexResult>,
}

impl<R: Read> Lexer<R> {
    pub fn lex(reader: R) -> Self {
        Lexer {
            input: CharReader::new(io::BufReader::new(reader)),
            position: Position::new(),
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
            if next_char.is_alphabetic() {
                ident.push(next_char);
            } else {
                self.ret_next(next_char);
                return None;
            }

            while let Some(next_char) = self.pop_next() {
                if next_char.is_alphanumeric() {
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

    fn parse_string(&mut self) -> Option<LexResult> {
        let mut buffer = String::new();
        let mut quote_closed = false;
        let quote_char = match self.pop_next() {
            Some(c) => c,
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
                                        return self.err(LexError::InvalidUnicodeEscape(uvalue as u32));
                                    },
                                };
                            } else {
                                return self.err(LexError::UnclosedStringError);
                            }
                        }

                        if let Some(next_char) = from_u32(uvalue as u32) {
                            buffer.push(next_char);
                        } else {
                            return self.err(LexError::InvalidUnicodeEscape(uvalue as u32));
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
}

impl<R: Read> Iterator for Lexer<R> {
    type Item = Result<LexToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked_next.is_some() {
            return self.peeked_next.take();
        }

        let mut option_char = self.pop_next();
        while option_char.is_some() && option_char.unwrap().is_whitespace() {
            option_char = self.pop_next();
        }

        if let Some(next_char) = option_char {
            if next_char == '{' {
                Some(Ok(LexToken::OpenBrace))
            } else if next_char == '}' {
                Some(Ok(LexToken::CloseBrace))
            } else if next_char == '[' {
                Some(Ok(LexToken::OpenBracket))
            } else if next_char == ']' {
                Some(Ok(LexToken::CloseBracket))
            } else if next_char == ',' {
                Some(Ok(LexToken::Comma))
            } else if next_char == '!' {
                Some(Ok(LexToken::Bang))
            } else if next_char == ':' {
                Some(Ok(LexToken::Colon))
            } else if next_char.is_alphabetic() {
                self.ret_next(next_char);
                self.parse_ident()
            } else if next_char.is_digit(10) || ['+', '-', '.'].contains(&next_char) {
                self.ret_next(next_char);
                self.parse_numeric()
            } else if next_char == '\"' || next_char == '\'' {
                self.ret_next(next_char);
                self.parse_string()
            } else {
                Some(Err(LexError::UnrecognisedCharError(next_char)))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::position::Position;
    use std::io::Cursor;

    #[test]
    fn iterator() {
        let cursor = Cursor::new("ident { } [ ! ] : , 34 3.5 'str' true".as_bytes());
        let mut lexer = Lexer::lex(cursor);
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::Identifier("ident".to_string()));
        assert_eq!(lexer.position, Position::at(0, 5));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::OpenBrace);
        assert_eq!(lexer.position, Position::at(0, 7));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::CloseBrace);
        assert_eq!(lexer.position, Position::at(0, 9));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::OpenBracket);
        assert_eq!(lexer.position, Position::at(0, 11));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::Bang);
        assert_eq!(lexer.position, Position::at(0, 13));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::CloseBracket);
        assert_eq!(lexer.position, Position::at(0, 15));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::Colon);
        assert_eq!(lexer.position, Position::at(0, 17));
        assert_eq!(lexer.next().unwrap().unwrap(), LexToken::Comma);
        assert_eq!(lexer.position, Position::at(0, 19));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::IntegerLit(34));
        assert_eq!(lexer.position, Position::at(0, 22));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::FloatLit(3.5));
        assert_eq!(lexer.position, Position::at(0, 26));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::StringLit("str".to_string()));
        assert_eq!(lexer.position, Position::at(0, 32));
        assert_eq!(lexer.next().unwrap().unwrap(),
            LexToken::Identifier("true".to_string()));
        assert_eq!(lexer.position, Position::at(0, 37));
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
}
