/// Represents a position in the file
#[derive(Debug, PartialEq)]
pub struct Position {
    /// line of the file
    pub line: usize,
    /// position in that line
    pub pos: usize,
}

impl Position {
    /// Construct a new position at `(0, 0)`
    pub fn new() -> Self {
        Self::at(0, 0)
    }

    /// Construct a new position at an arbitrary line and position
    pub fn at(line: usize, pos: usize) -> Self {
        Position { line: line, pos: pos }
    }
}

#[derive(Debug)]
pub struct MutablePosition {
    pub line: usize,
    pub pos: usize,
    line_lengths: Vec<usize>,
}

impl MutablePosition {
    pub fn new() -> Self {
        Self::at(0, 0)
    }
    pub fn at(line: usize, pos: usize) -> Self {
        MutablePosition { line: line, pos: pos, line_lengths: Vec::new() }
    }
    pub fn new_line(&mut self) -> &mut Self {
        self.line_lengths.push(self.pos);
        self.pos = 0;
        self.line += 1;
        self
    }
    pub fn push(&mut self, amt: usize) -> &mut Self {
        self.pos += amt;
        self
    }

    pub fn unpush(&mut self, amt: usize) -> &mut Self {
        if amt <= self.pos {
            self.pos -= amt;
        } else {
            if let Some(length) = self.line_lengths.pop() {
                let oldpos = self.pos;
                self.pos = length;
                self.line -= 1;
                self.unpush(amt - oldpos - 1);
            } else {
                panic!("Cannot unpush any further - no previous history");
            }
        }

        self
    }

    pub fn freeze(self) -> Position {
        Position { line: self.line, pos: self.pos }
    }
}

impl Clone for MutablePosition {
    fn clone(&self) -> Self {
        MutablePosition { line: self.line, pos: self.pos, line_lengths: Vec::new() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.line = source.line;
        self.pos = source.pos;
        self.line_lengths = Vec::new();
    }
}

impl PartialEq for MutablePosition {
    fn eq(&self, other: &MutablePosition) -> bool {
        self.pos == other.pos && self.line == other.line
    }

    fn ne(&self, other: &MutablePosition) -> bool {
        self.pos != other.pos || self.line != other.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let pos = MutablePosition::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);
    }

    #[test]
    fn push_position() {
        let mut pos = MutablePosition::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);

        pos.push(1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 1);

        pos.push(5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 6);
    }

    #[test]
    fn new_line() {
        let mut pos = MutablePosition::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);

        pos.new_line();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.pos, 0);

        pos.push(10);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.pos, 10);

        pos.new_line();
        assert_eq!(pos.line, 2);
        assert_eq!(pos.pos, 0);
    }

    #[test]
    fn unpush() {
        let mut pos = MutablePosition::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);

        pos.push(10);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 10);

        pos.unpush(1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 9);

        pos.push(1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 10);

        pos.new_line();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.pos, 0);

        pos.unpush(1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 10);
    }

    #[test]
    fn equality() {
        let mut pos = MutablePosition::new();
        assert_eq!(pos, MutablePosition::new());
        assert_eq!(pos, MutablePosition::at(0, 0));

        pos.push(15);
        assert_eq!(pos, *MutablePosition::new().push(15));
        assert_eq!(pos, MutablePosition::at(0, 15));

        pos.new_line();
        assert_eq!(pos, *MutablePosition::new().new_line());
        assert_eq!(pos, MutablePosition::at(1, 0));
    }
}

