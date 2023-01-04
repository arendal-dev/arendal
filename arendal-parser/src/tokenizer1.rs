use super::{NewLine, Pos};
use arendal_error::{Errors, Result};

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    Tokenizer::new(input).tokenize()
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pos: Pos, // Starting position of the token
    token_type: TokenType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType<'a> {
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
    OpenParens,
    CloseParens,
    OpenCBracket,
    CloseCBracket,
    OpenSBracket,
    CloseSBracket,
    Underscore,
    Digits(&'a str),
    Word(&'a str),
}

impl<'a> TokenType<'a> {
    fn single(c: char) -> Option<TokenType<'a>> {
        match c {
            '+' => Some(TokenType::Plus),
            '-' => Some(TokenType::Minus),
            '*' => Some(TokenType::Star),
            '/' => Some(TokenType::Slash),
            '.' => Some(TokenType::Dot),
            '>' => Some(TokenType::Greater),
            '<' => Some(TokenType::Less),
            '!' => Some(TokenType::Bang),
            '=' => Some(TokenType::Equal),
            '(' => Some(TokenType::OpenParens),
            ')' => Some(TokenType::CloseParens),
            '{' => Some(TokenType::OpenCBracket),
            '}' => Some(TokenType::CloseCBracket),
            '[' => Some(TokenType::OpenSBracket),
            ']' => Some(TokenType::CloseSBracket),
            '_' => Some(TokenType::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    input: &'a str,
    chars: Vec<char>,
    tokens: Vec<Token<'a>>,
    errors: Errors,
    // Positions
    pos: Pos,
    // Current char index from the beginning of the input
    char_index: usize,
    // Start of the current token
    token_start: Pos,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
        Tokenizer {
            input,
            chars: input.chars().collect(),
            tokens: Vec::new(),
            errors: Errors::new(),
            pos: Pos::new(),
            char_index: 0,
            token_start: Pos::new(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    // Consumes one char, advancing the indices accordingly.
    fn consume(&mut self) {
        let bytes = self.chars[self.char_index].len_utf8();
        self.pos = self.pos.advance(bytes);
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

    fn tokenize(mut self) -> Result<Vec<Token<'a>>> {
        while !self.is_done() {
            self.token_start = self.pos;
            let c = self.peek();
            if let Some(t) = TokenType::single(c) {
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
    fn consume_single_char_token(&mut self, token_type: TokenType<'a>) {
        self.consume();
        self.add_token(token_type);
    }

    // Consumes a new line if found in the current position
    fn consume_eol(&mut self, c: char) -> bool {
        let newline: Option<NewLine>;
        if c == '\n' {
            newline = Some(NewLine::LF);
        } else if c == '\r' && self.next_matches('\n') {
            newline = Some(NewLine::CRLF);
        } else {
            newline = None;
        }
        match newline {
            Some(nl) => {
                self.pos = self.pos.newline(nl);
                self.char_index += nl.bytes();
                self.add_token(TokenType::EndOfLine(nl));
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
        let token = TokenType::Spaces(self.consume_multiple(' '));
        self.add_token(token);
    }

    fn consume_tabs(&mut self) {
        let token = TokenType::Tabs(self.consume_multiple('\t'));
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
            self.add_token(TokenType::Digits(self.get_token_str()));
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
            self.add_token(TokenType::Word(self.get_token_str()));
        }
        consumed
    }

    fn get_token_str(&self) -> &'a str {
        &self.input[self.token_start.index..self.pos.index]
    }

    fn add_token(&mut self, token_type: TokenType<'a>) {
        self.tokens.push(Token {
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
    use super::{Token, TokenType};
    use crate::{NewLine, Pos};
    use NewLine::*;

    struct TestCase<'a> {
        pos: Pos,
        tokens: Vec<Token<'a>>,
    }

    impl<'a> TestCase<'a> {
        fn new() -> TestCase<'a> {
            TestCase {
                pos: Pos::new(),
                tokens: Vec::new(),
            }
        }

        fn newline(mut self, nl: NewLine) -> Self {
            self.tokens.push(Token {
                pos: self.pos,
                token_type: TokenType::EndOfLine(nl),
            });
            self.pos = self.pos.newline(nl);
            self
        }

        fn token(mut self, token_type: TokenType<'a>, bytes: usize) -> Self {
            if let TokenType::EndOfLine(_) = token_type {
                assert!(false);
            }
            self.tokens.push(Token {
                pos: self.pos,
                token_type,
            });
            self.pos = self.pos.advance(bytes);
            self
        }

        fn single(self, token_type: TokenType<'a>) -> Self {
            self.token(token_type, 1)
        }

        fn spaces(self, n: usize) -> Self {
            self.token(TokenType::Spaces(n), n)
        }

        fn tabs(self, n: usize) -> Self {
            self.token(TokenType::Tabs(n), n)
        }

        fn digits(self, digits: &'a str) -> Self {
            self.token(TokenType::Digits(digits), digits.len())
        }

        fn word(self, word: &'a str) -> Self {
            self.token(TokenType::Word(word), word.len())
        }

        fn ok(&self, input: &str) {
            match super::tokenize(input) {
                Ok(tokens) => assert_eq!(tokens, self.tokens),
                Err(_) => assert!(false),
            }
        }
    }

    #[test]
    fn empty() {
        TestCase::new().ok("");
    }

    #[test]
    fn spaces() {
        TestCase::new().spaces(3).ok("   ");
    }

    #[test]
    fn tabs() {
        TestCase::new().tabs(3).ok("\t\t\t");
    }

    #[test]
    fn lf() {
        TestCase::new().newline(LF).ok("\n");
    }

    #[test]
    fn crlf() {
        TestCase::new().newline(CRLF).ok("\r\n");
    }

    #[test]
    fn singles() {
        TestCase::new()
            .single(TokenType::Plus)
            .single(TokenType::Minus)
            .single(TokenType::Star)
            .single(TokenType::Dot)
            .single(TokenType::Slash)
            .single(TokenType::Greater)
            .single(TokenType::Less)
            .single(TokenType::Bang)
            .single(TokenType::OpenParens)
            .single(TokenType::CloseParens)
            .single(TokenType::Equal)
            .single(TokenType::OpenCBracket)
            .single(TokenType::CloseCBracket)
            .single(TokenType::OpenSBracket)
            .single(TokenType::CloseSBracket)
            .single(TokenType::Underscore)
            .ok("+-*./><!()={}[]_");
    }

    #[test]
    fn digits() {
        TestCase::new().digits("1234").ok("1234");
    }

    #[test]
    fn word() {
        TestCase::new().word("abc").ok("abc");
    }

    #[test]
    fn harness() {
        TestCase::new().spaces(3).newline(LF).tabs(1).ok("   \n\t");
    }
}
