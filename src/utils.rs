use std::io::prelude::*;
use std::collections::VecDeque;

pub fn ident_head(c: char) -> bool {
    // TODO: This is ugly.  This should be done *waaaaay* better.
    match c {
        'a'...'z' => true,
        'A'...'Z' => true,
        '_' => true,
        '\u{00A8}' | '\u{00AA}' | '\u{00AD}' | '\u{00AF}' => true,
        '\u{00B2}'...'\u{00B5}' | '\u{00B7}'...'\u{00BA}' => true,
        '\u{00BC}'...'\u{00BE}' | '\u{00C0}'...'\u{00D6}' => true,
        '\u{00D8}'...'\u{00F6}' | '\u{00F8}'...'\u{00FF}' => true,
        '\u{0100}'...'\u{02FF}' | '\u{0370}'...'\u{167F}' => true,
        '\u{1681}'...'\u{180D}' | '\u{180F}'...'\u{1DBF}' => true,
        '\u{1E00}'...'\u{1FFF}' | '\u{200B}'...'\u{200D}' => true,
        '\u{202A}'...'\u{202E}' | '\u{203F}'...'\u{2040}' => true,
        '\u{2054}' | '\u{2060}'...'\u{206F}' => true,
        '\u{2070}'...'\u{20CF}' | '\u{2100}'...'\u{218F}' => true,
        '\u{2460}'...'\u{24FF}' | '\u{2776}'...'\u{2793}' => true,
        '\u{2C00}'...'\u{2DFF}' | '\u{2E80}'...'\u{2FFF}' => true,
        '\u{3004}'...'\u{3007}' | '\u{3021}'...'\u{302F}' => true,
        '\u{3031}'...'\u{303F}' | '\u{3040}'...'\u{D7FF}' => true,
        '\u{F900}'...'\u{FD3D}' | '\u{FD40}'...'\u{FDCF}' => true,
        '\u{FDF0}'...'\u{FE1F}' | '\u{FE30}'...'\u{FE45}' => true,
        '\u{FE47}'...'\u{FFFD}' => true,
        '\u{10000}'...'\u{1FFFD}' | '\u{20000}'...'\u{2FFFD}' => true,
        '\u{30000}'...'\u{3FFFD}' | '\u{40000}'...'\u{4FFFD}' => true,
        '\u{50000}'...'\u{5FFFD}' | '\u{60000}'...'\u{6FFFD}' => true,
        '\u{70000}'...'\u{7FFFD}' | '\u{80000}'...'\u{8FFFD}' => true,
        '\u{90000}'...'\u{9FFFD}' | '\u{A0000}'...'\u{AFFFD}' => true,
        '\u{B0000}'...'\u{BFFFD}' | '\u{C0000}'...'\u{CFFFD}' => true,
        '\u{D0000}'...'\u{DFFFD}' | '\u{E0000}'...'\u{EFFFD}' => true,
        _ => false
    }
}

pub fn ident_body(c: char) -> bool {
    if !ident_head(c) {
        match c {
            '0'...'9' => true,
            '\u{0300}'...'\u{036F}' | '\u{1DC0}'...'\u{1DFF}' => true,
            '\u{20D0}'...'\u{20FF}' | '\u{FE20}'...'\u{FE2F}' => true,
            _ => false
        }
    } else {
        true
    }
}

pub struct CharReader<R: BufRead> {
    reader: R,
    buffer: VecDeque<char>,
}

impl<R: BufRead> CharReader<R> {
    pub fn new(reader: R) -> Self {
        CharReader {
            reader: reader,
            buffer: VecDeque::new(),
        }
    }
}

impl<R: BufRead> Iterator for CharReader<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffer.is_empty() {
            let mut str_buffer = String::new();
            match self.reader.read_line(&mut str_buffer) {
                Ok(n) => if n == 0 { return None; },
                Err(_) => { return None; },
            }
            self.buffer = str_buffer.chars().collect();
        }

        self.buffer.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::CharReader;
    use std::io::{Cursor, empty};

    #[test]
    fn iteration() {
        let mut reader = CharReader::new(Cursor::new("text".as_bytes()));
        assert_eq!(reader.next(), Some('t'));
        assert_eq!(reader.next(), Some('e'));
        assert_eq!(reader.next(), Some('x'));
        assert_eq!(reader.next(), Some('t'));
        assert_eq!(reader.next(), None);
    }

    #[test]
    fn empty_iteration() {
        let mut reader = CharReader::new(empty());
        assert_eq!(reader.next(), None);
    }
}
