// This struct represents an input string and a byte index in it. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos<'a> {
    input: &'a str, // Input string
    index: usize, // Byte index from the beginning of the input
}

impl<'a> Pos<'a> {
    // Creates a new position at the beginning of the input
    pub fn new(input: &str) -> Pos {
        Pos {
            input,
            index: 0,
        }
    }

    // Returns true if we have reached the end of the input
    pub fn is_done(&self) -> bool {
        self.index >= self.input.len()
    }
    

    // Advances the current position the provided number of bytes
    pub fn advance(&mut self, bytes: usize) {
        self.index += bytes;
    }

    // Resets the current position
    pub fn reset(&mut self) {
        self.index = 0;
    }

    // Returns a slice from the current position to provided one
    // Panics if the positions are for different input or if the end index is smaller
    // than the current one or larger than the length of the input.
    pub fn str_to(&self, to: &Pos) -> &'a str {
        assert_eq!(self.input, to.input);
        assert!(self.index <= to.index);
        &self.input[self.index..to.index]
    }
}

#[cfg(test)]
mod tests {
    use super::Pos;

    #[test]
    fn harness() {
        let mut pos = Pos::new("1234");
        assert_eq!(0, pos.index);
        assert!(!pos.is_done());
        pos.advance(2);
        assert_eq!(2, pos.index);
        assert!(!pos.is_done());
        pos.advance(3);
        assert_eq!(5, pos.index);
        assert!(pos.is_done());
        pos.reset();
        assert_eq!(0, pos.index);
        assert!(!pos.is_done());
    }


}
