use arcstr::{ArcStr, Substr};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringInput {
    input: ArcStr,
}

impl StringInput {
    pub fn from_str(input: &str) -> StringInput {
        StringInput {
            input: input.into(),
        }
    }

    pub fn from_arcstr(input: ArcStr) -> StringInput {
        StringInput { input }
    }

    pub fn as_char_vec(&self) -> Vec<char> {
        self.input.chars().collect()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StrLen {
    bytes: usize,
    chars: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    line_number: usize, // starts with 1
    bytes: usize,       // bytes from the beginning of the string to the beginning of the line
}

impl Line {
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

impl Default for Line {
    fn default() -> Self {
        Line {
            line_number: 1,
            bytes: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrPos {
    bytes: usize, // bytes from the beginning of the string
    line: Line,
    column: usize, // chars from the beginning of the line
}

impl Default for StrPos {
    fn default() -> Self {
        StrPos {
            bytes: 0,
            line: Line::default(),
            column: 1,
        }
    }
}

impl StrPos {
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn line(&self) -> Line {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

#[derive(Debug)]
pub enum StrRangeError {
    DifferentInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrRange {
    input: StringInput,
    from: StrPos,
    to: StrPos,
}

// Merges two ranges. Assumptions:
// - Both ranges are valid and from the same input.
// - r1.from.from_start_bytes <= r2.from.from_start_bytes
fn merge_ranges(r1: &StrRange, r2: &StrRange) -> StrRange {
    if r1.to.bytes >= r2.to.bytes {
        r1.clone()
    } else {
        StrRange {
            input: r1.input.clone(),
            from: r1.from,
            to: r2.to,
        }
    }
}

impl StrRange {
    pub fn new(input: StringInput) -> Self {
        StrRange {
            input,
            from: StrPos::default(),
            to: StrPos::default(),
        }
    }

    pub fn from(&self) -> &StrPos {
        &self.from
    }

    pub fn to(&self) -> &StrPos {
        &self.to
    }

    pub fn catch_up(&mut self) {
        self.from = self.to;
    }

    // Advance one char
    // TODO: check if it is the right char, at least in debug mode.
    pub fn advance(&mut self, c: char) {
        self.to.bytes += c.len_utf8();
        if c == '\n' {
            self.to.line.line_number += 1;
            self.to.line.bytes = self.to.bytes;
            self.to.column = 1;
        } else {
            self.to.column += 1;
        }
    }

    // Returns the substring represented by this range
    pub fn substr(&self) -> Substr {
        let from = self.from.bytes;
        let to = self.to.bytes;
        self.input.input.substr(from..to)
    }

    pub fn merge(&self, other: &StrRange) -> Result<StrRange, StrRangeError> {
        if self.input != other.input {
            Err(StrRangeError::DifferentInput)
        } else if self.from.bytes <= other.from.bytes {
            Ok(merge_ranges(self, other))
        } else {
            Ok(merge_ranges(other, self))
        }
    }
}

impl fmt::Display for StrRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = self.from.bytes;
        let to = self.to.bytes;
        if from == to {
            write!(f, "{}", from)
        } else {
            write!(f, "{}-{}", from, to)
        }
    }
}
