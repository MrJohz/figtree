#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub pos: usize,
}

impl Position {
    pub fn new() -> Self {
        Self::at(0, 0)
    }
    pub fn at(line: usize, pos: usize) -> Self {
        Position { line: line, pos: pos }
    }
    pub fn new_line(self) -> Self {
        Position { line: self.line + 1, pos: 0 }
    }
    pub fn push(self, amt: usize) -> Self {
        Position { line: self.line, pos: self.pos + amt }
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn construction() {
        let pos = Position::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);
    }

    #[test]
    fn push_position() {
        let pos = Position::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);

        let pos = pos.push(1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 1);

        let pos = pos.push(5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 6);
    }

    #[test]
    fn new_line() {
        let pos = Position::new();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.pos, 0);

        let pos = pos.new_line();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.pos, 0);

        let pos = pos.push(10);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.pos, 10);

        let pos = pos.new_line();
        assert_eq!(pos.line, 2);
        assert_eq!(pos.pos, 0);
    }
}

