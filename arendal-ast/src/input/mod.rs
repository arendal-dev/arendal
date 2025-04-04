use arcstr::{ArcStr, Substr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringInput {
    input: ArcStr
}

impl StringInput {
    pub fn from_str(input: &str) -> StringInput {
        StringInput { input: input.into() }
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

impl StrLen {
    fn validate(bytes: usize, chars: usize) {
        if chars < bytes {
            panic!("A char takes at least one byte");
        } 
    }

    pub fn new(bytes: usize, chars: usize) -> StrLen {
        Self::validate(bytes, chars);
        StrLen { bytes, chars }
    }

    pub fn of_str(str: &str) -> StrLen {
        Self::new(str.len(), str.chars().count())
    }

    pub fn of_char(c: char) -> StrLen {
        Self::new(c.len_utf8(), 1)
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn chars(&self) -> usize {
        self.chars
    }

    pub fn add(&mut self, other: StrLen) {
        self.bytes += other.bytes;
        self.chars += other.chars;
    }

    pub fn add_char(&mut self, other: char) {
        self.bytes += other.len_utf8();
        self.chars += 1;
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewLine {
    LF,
    CRLF,
}

impl NewLine {
    pub fn len(self) -> StrLen {
        let len = match self {
            Self::LF => 1,
            Self::CRLF => 2,
        };
        StrLen::new(len, len)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    line_number: usize,
    start_bytes: usize
}

impl Line {
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn start_bytes(&self) -> usize {
        self.start_bytes
    }
}

impl Default for Line {
    fn default() -> Self {
        Line { line_number: 1, start_bytes: 0 }
    }  
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StrPos {
    line: Line,
    from_start_bytes: usize,
    from_line_chars: usize,
}

impl StrPos {
    pub fn line(&self) -> Line {
        self.line
    }

    pub fn from_start_bytes(&self) -> usize {
        self.from_start_bytes
    }

    pub fn from_line_chars(&self) -> usize {
        self.from_line_chars
    }
}


pub enum StrRangeError {
    DifferentInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrRange {
    input: StringInput,
    from: StrPos,
    to: StrPos
}

// Merges two ranges. Assumptions:
// - Both ranges are valid and from the same input.
// - r1.from.from_start_bytes <= r2.from.from_start_bytes
fn merge_ranges(r1: &StrRange, r2: &StrRange) -> StrRange {
    if r1.to.from_start_bytes >= r2.to.from_start_bytes {
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

    pub fn from_bytes(&self) -> usize {
        self.from.from_start_bytes
    }

    pub fn catch_up(&mut self) {
        self.from = self.to;
    } 

    pub fn new_line(&mut self, new_line: NewLine) {
        self.to.from_start_bytes += new_line.len().bytes;
        self.to.line.line_number += 1;
        self.to.line.start_bytes = self.to.from_start_bytes;
        self.to.from_line_chars = 0;
    }

    // Advance the to position in the same line
    pub fn advance(&mut self, len: StrLen) {
        self.to.from_start_bytes += len.bytes;
        self.to.from_line_chars += len.chars;
    }

    // Returns a substring starting in the from position of this range
    pub fn get_substr_len(&self, len: StrLen) -> Substr {
        let from = self.from.from_start_bytes;
        let to = from + len.bytes;
        self.input.input.substr(from..to)
    }

    // Returns the substring represented by this range
    pub fn substr(&self) -> Substr {
        let from = self.from.from_start_bytes;
        let to = self.to.from_start_bytes;
        self.input.input.substr(from..to)
    }

    pub fn merge(&self, other: &StrRange) -> Result<StrRange, StrRangeError> {
        if self.input != other.input {
            Err(StrRangeError::DifferentInput)
        } else if self.from.from_start_bytes <= other.from.from_start_bytes {
            Ok(merge_ranges(self, other))
        } else {
            Ok(merge_ranges(other, self))
        }
    }


}


