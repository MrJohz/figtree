use std::io::prelude::*;
use std::collections::VecDeque;

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
