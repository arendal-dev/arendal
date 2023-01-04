use super::{NewLine, Pos};
use arendal_error::{Errors, Result};
use num::BigInt;

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    Tokenizer::new(input).tokenize()
}

#[derive(Debug)]
pub struct Token {
    pos: Pos, // Starting position of the token
    token_type: TokenType,
}

#[derive(Debug)]
enum TokenType {
    Spaces(usize),
    Tabs(usize),
    EndOfLine(NewLine),
    Natural(BigInt),
}

impl Token {}

struct Tokenizer<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    tokens: Vec<Token>,
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
            chars: input.char_indices().collect(),
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
        let bytes = self.chars[self.char_index].0;
        self.pos = self.pos.advance(bytes);
        self.char_index += 1;
    }

    // Returns the char at the current index.
    // Panics if we have reached the end of the input
    fn peek(&self) -> char {
        self.chars[self.char_index].1
    }

    // Returns true if there's a next char and it's equal to the provided one.
    fn next_matches(&self, c: char) -> bool {
        let i = self.char_index + 1;
        if i >= self.chars.len() {
            false
        } else {
            c == self.chars[i].1
        }
    }

    /*
    // Consumes one char if we are not at the end of the input
    // and the current char is the same as the provided.
    fn consume_if_char(&mut self, c: char) -> bool {
        if !self.is_done() && c == self.peek() {
            self.consume();
            true
        } else {
            false
        }
    }

    // Consumes two chars if we are at least two chars from the end of the input
    // and the current and next ones are same as the provided.
    fn consume_if_two_chars(&mut self, c1: char, c2: char) -> bool {
        let i = self.char_index + 1;
        if i >= self.chars.len() {
            if c1 == self.peek() && c2 == self.chars[i].1 {
                self.consume();
                self.consume();
                return true;
            }
        }
        false
    }

    // Starts a token in the current char, consuming it.
    // Only used for variable-length tokens.
    fn start_token(&mut self) {
        self.token_index = self.index;
        self.token_line_index = self.line_index;
        self.consume();
    }
    */

    fn tokenize(mut self) -> Result<Vec<Token>> {
        while !self.is_done() {
            match self.peek() {
                ' ' => self.consume_spaces(),
                _ => self.add_unexpected_char(self.peek()),
            }
        }
        self.errors.to_result(self.tokens)
    }

    // Consumes a new line if found in the current position
    fn consume_eol(&mut self) {
        let c = self.peek();
        if c == '\n' {}
    }

    // Starts a token a consumes chars while they are equal to the one provided.
    // Returns the number of chars consumed.
    fn consume_multiple(&mut self, c: char) -> usize {
        let mut count = 1;
        while self.peek() == c {
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

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token {
            pos: self.token_start,
            token_type,
        });
    }

    fn add_indentation_error(&mut self) {
        self.errors.add(super::indentation_error(self.pos))
    }

    fn add_unexpected_char(&mut self, c: char) {
        self.errors.add(super::unexpected_char(self.pos, c))
    }
}
