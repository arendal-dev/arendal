mod pass2;

use super::{Errors, Pos, Result};

fn tokenize(input: &str) -> Result<CharTokens> {
    Tokenizer::new(input).tokenize()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewLine {
    LF,
    CRLF,
}

impl NewLine {
    fn bytes(self) -> usize {
        match self {
            Self::LF => 1,
            Self::CRLF => 2,
        }
    }

    fn chars(self) -> usize {
        self.bytes() // we have another method in case it's different in the future
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}

type CharTokens<'a> = Vec<CharToken<'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CharToken<'a> {
    pos: Pos<'a>, // Starting position of the token
    token_type: CharTokenType<'a>,
}

impl<'a> CharToken<'a> {
    fn is_whitespace(&self) -> bool {
        self.token_type.is_whitespace()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CharTokenType<'a> {
    Spaces(usize),
    Tabs(usize),
    EndOfLine(NewLine),
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Greater,
    Less,
    Bang,
    Equal,
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Digits(&'a str),
    Word(&'a str),
}

impl<'a> CharTokenType<'a> {
    fn is_whitespace(&self) -> bool {
        match self {
            CharTokenType::Spaces(_) | CharTokenType::Tabs(_) => true,
            _ => false,
        }
    }

    fn single(c: char) -> Option<CharTokenType<'a>> {
        match c {
            '+' => Some(CharTokenType::Plus),
            '-' => Some(CharTokenType::Minus),
            '*' => Some(CharTokenType::Star),
            '/' => Some(CharTokenType::Slash),
            '.' => Some(CharTokenType::Dot),
            '>' => Some(CharTokenType::Greater),
            '<' => Some(CharTokenType::Less),
            '!' => Some(CharTokenType::Bang),
            '=' => Some(CharTokenType::Equal),
            '(' => Some(CharTokenType::Open(Enclosure::Parens)),
            ')' => Some(CharTokenType::Close(Enclosure::Parens)),
            '{' => Some(CharTokenType::Open(Enclosure::Curly)),
            '}' => Some(CharTokenType::Close(Enclosure::Curly)),
            '[' => Some(CharTokenType::Open(Enclosure::Square)),
            ']' => Some(CharTokenType::Close(Enclosure::Square)),
            '_' => Some(CharTokenType::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    chars: Vec<char>,
    tokens: CharTokens<'a>,
    errors: Errors<'a>,
    // Positions
    pos: Pos<'a>,
    // Current char index from the beginning of the input
    char_index: usize,
    // Start of the current token
    token_start: Pos<'a>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
        Tokenizer {
            chars: input.chars().collect(),
            tokens: Vec::new(),
            errors: Errors::new(),
            pos: Pos::new(input),
            char_index: 0,
            token_start: Pos::new(input),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    // Consumes one char, advancing the indices accordingly.
    fn consume(&mut self) {
        let bytes = self.chars[self.char_index].len_utf8();
        self.pos.advance(bytes);
        self.char_index += 1;
    }

    // Returns the char at the current index.
    // Panics if we have reached the end of the input
    fn peek(&self) -> char {
        self.chars[self.char_index]
    }

    // Returns true if there's a next char and it's equal to the provided one.
    fn next_matches(&self, c: char) -> bool {
        let i = self.char_index + 1;
        if i >= self.chars.len() {
            false
        } else {
            c == self.chars[i]
        }
    }

    fn tokenize(mut self) -> Result<'a, CharTokens<'a>> {
        while !self.is_done() {
            self.token_start = self.pos;
            let c = self.peek();
            if let Some(t) = CharTokenType::single(c) {
                self.consume_single_char_token(t);
            } else {
                match c {
                    ' ' => self.consume_spaces(),
                    '\t' => self.consume_tabs(),
                    _ => self.tokenize2(c),
                }
            }
        }
        self.errors.to_result(self.tokens)
    }

    fn tokenize2(&mut self, c: char) {
        if !self.consume_eol(c) && !self.consume_digits(c) && !self.consume_word(c) {
            self.add_unexpected_char(c)
        }
    }

    // Consumes a char, creating the provided token
    fn consume_single_char_token(&mut self, token_type: CharTokenType<'a>) {
        self.consume();
        self.add_token(token_type);
    }

    // Returns whether the provided char is a newline, peeking the next if needed
    // Consumes a new line if found in the current position
    fn is_eol(&mut self, c: char) -> Option<NewLine> {
        if c == '\n' {
            Some(NewLine::LF)
        } else if c == '\r' && self.next_matches('\n') {
            Some(NewLine::CRLF)
        } else {
            None
        }
    }

    // Consumes a new line if found in the current position
    fn consume_eol(&mut self, c: char) -> bool {
        match self.is_eol(c) {
            Some(nl) => {
                self.pos.advance(nl.bytes());
                self.char_index += nl.chars();
                self.add_token(CharTokenType::EndOfLine(nl));
                true
            }
            _ => false,
        }
    }

    // Starts a token a consumes chars while they are equal to the one provided.
    // Returns the number of chars consumed.
    fn consume_multiple(&mut self, c: char) -> usize {
        let mut count = 1;
        self.consume();
        while !self.is_done() && self.peek() == c {
            self.consume();
            count += 1
        }
        count
    }

    fn consume_spaces(&mut self) {
        let token = CharTokenType::Spaces(self.consume_multiple(' '));
        self.add_token(token);
    }

    fn consume_tabs(&mut self) {
        let token = CharTokenType::Tabs(self.consume_multiple('\t'));
        self.add_token(token);
    }

    fn consume_digits(&mut self, mut c: char) -> bool {
        let mut consumed = false;
        while c.is_ascii_digit() {
            self.consume();
            consumed = true;
            if self.is_done() {
                break;
            } else {
                c = self.peek();
            }
        }
        if consumed {
            self.add_token(CharTokenType::Digits(self.get_token_str()));
        }
        consumed
    }

    fn consume_word(&mut self, mut c: char) -> bool {
        if !c.is_ascii_alphabetic() {
            return false;
        }
        let mut consumed = false;
        while c.is_ascii_alphanumeric() {
            self.consume();
            consumed = true;
            if self.is_done() {
                break;
            } else {
                c = self.peek();
            }
        }
        if consumed {
            self.add_token(CharTokenType::Word(self.get_token_str()));
        }
        consumed
    }

    fn get_token_str(&self) -> &'a str {
        self.token_start.str_to(&self.pos)
    }

    fn add_token(&mut self, token_type: CharTokenType<'a>) {
        self.tokens.push(CharToken {
            pos: self.token_start,
            token_type,
        });
    }

    fn add_unexpected_char(&mut self, c: char) {
        self.errors.add(super::unexpected_char(self.pos, c))
    }
}

#[cfg(test)]
mod tests {
    use super::{CharToken, CharTokenType, CharTokens, Enclosure, NewLine, Pos};
    use NewLine::*;

    struct TestCase<'a> {
        input: &'a str,
        pos: Pos<'a>,
        tokens: CharTokens<'a>,
    }

    impl<'a> TestCase<'a> {
        fn new(input: &'a str) -> TestCase<'a> {
            TestCase {
                input,
                pos: Pos::new(input),
                tokens: Vec::new(),
            }
        }

        fn newline(mut self, nl: NewLine) -> Self {
            self.tokens.push(CharToken {
                pos: self.pos,
                token_type: CharTokenType::EndOfLine(nl),
            });
            self.pos.advance(nl.bytes());
            self
        }

        fn token(mut self, token_type: CharTokenType<'a>, bytes: usize) -> Self {
            if let CharTokenType::EndOfLine(_) = token_type {
                assert!(false);
            }
            self.tokens.push(CharToken {
                pos: self.pos,
                token_type,
            });
            self.pos.advance(bytes);
            self
        }

        fn single(self, token_type: CharTokenType<'a>) -> Self {
            self.token(token_type, 1)
        }

        fn spaces(self, n: usize) -> Self {
            self.token(CharTokenType::Spaces(n), n)
        }

        fn tabs(self, n: usize) -> Self {
            self.token(CharTokenType::Tabs(n), n)
        }

        fn digits(self, digits: &'a str) -> Self {
            self.token(CharTokenType::Digits(digits), digits.len())
        }

        fn word(self, word: &'a str) -> Self {
            self.token(CharTokenType::Word(word), word.len())
        }

        fn ok(&self) {
            match super::tokenize(self.input) {
                Ok(tokens) => assert_eq!(tokens, self.tokens),
                Err(_) => assert!(false),
            }
        }
    }

    #[test]
    fn empty() {
        TestCase::new("").ok();
    }

    #[test]
    fn spaces() {
        TestCase::new("   ").spaces(3).ok();
    }

    #[test]
    fn tabs() {
        TestCase::new("\t\t\t").tabs(3).ok();
    }

    #[test]
    fn lf() {
        TestCase::new("\n").newline(LF).ok();
    }

    #[test]
    fn crlf() {
        TestCase::new("\r\n").newline(CRLF).ok();
    }

    #[test]
    fn singles() {
        TestCase::new("+-*./><!()={}[]_")
            .single(CharTokenType::Plus)
            .single(CharTokenType::Minus)
            .single(CharTokenType::Star)
            .single(CharTokenType::Dot)
            .single(CharTokenType::Slash)
            .single(CharTokenType::Greater)
            .single(CharTokenType::Less)
            .single(CharTokenType::Bang)
            .single(CharTokenType::Open(Enclosure::Parens))
            .single(CharTokenType::Close(Enclosure::Parens))
            .single(CharTokenType::Equal)
            .single(CharTokenType::Open(Enclosure::Curly))
            .single(CharTokenType::Close(Enclosure::Curly))
            .single(CharTokenType::Open(Enclosure::Square))
            .single(CharTokenType::Close(Enclosure::Square))
            .single(CharTokenType::Underscore)
            .ok();
    }

    #[test]
    fn digits() {
        TestCase::new("1234").digits("1234").ok();
        TestCase::new("12 34")
            .digits("12")
            .spaces(1)
            .digits("34")
            .ok();
    }

    #[test]
    fn word() {
        TestCase::new("abc").word("abc").ok();
        TestCase::new("abc4e").word("abc4e").ok();
        TestCase::new("4bc5e").digits("4").word("bc5e").ok();
    }

    #[test]
    fn harness() {
        TestCase::new("   \n\t").spaces(3).newline(LF).tabs(1).ok();
    }
}
