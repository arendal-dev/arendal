use super::Enclosure;
use crate::error::{Error, Errors, Loc, Result, L};
use crate::{ArcStr, Substr};

pub(super) fn tokenize(input: &str) -> Result<Tokens> {
    Tokenizer::new(ArcStr::from(input)).tokenize()
}

pub(super) type Tokens = Vec<L<Token>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NewLine {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token {
    Spaces(usize),
    Tabs(usize),
    EndOfLine(NewLine),
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Greater,
    GreaterOrEq,
    Less,
    LessOrEq,
    Bang,
    Assignment,
    Equals,
    NotEquals,
    Ampersand,
    LogicalAnd,
    Pipe,
    LogicalOr,
    Colon,
    DoubleColon,
    Open(Enclosure),
    Close(Enclosure),
    Underscore,
    Digits(Substr),
    Word(Substr),
}

impl Token {
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self,
            Token::Spaces(_) | Token::Tabs(_) | Token::EndOfLine(_)
        )
    }

    fn chars(&self) -> usize {
        match self {
            Token::EndOfLine(nl) => nl.chars(),
            Token::Digits(s) => s.chars().count(),
            Token::Word(s) => s.chars().count(),
            _ => self.bytes(),
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Token::Spaces(n) => *n,
            Token::Tabs(n) => *n,
            Token::EndOfLine(nl) => nl.bytes(),
            Token::Digits(s) => s.len(),
            Token::Word(s) => s.len(),
            Token::NotEquals => 2,
            Token::LogicalAnd => 2,
            Token::LogicalOr => 2,
            Token::GreaterOrEq => 2,
            Token::LessOrEq => 2,
            Token::DoubleColon => 2,
            _ => 1,
        }
    }
}

struct Tokenizer {
    chars: Vec<char>,
    tokens: Tokens,
    input: ArcStr,
    byte_index: usize, // Current byte index from the beginning of the input
    char_index: usize, // Current char index from the beginning of the input
}

impl Tokenizer {
    fn new(input: ArcStr) -> Tokenizer {
        Tokenizer {
            chars: input.chars().collect(),
            tokens: Default::default(),
            input,
            byte_index: 0,
            char_index: 0,
        }
    }

    fn loc(&self) -> Loc {
        Loc::input(self.input.clone(), self.byte_index)
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.char_index >= self.chars.len()
    }

    // Consumes one char, advancing the indices accordingly.
    fn consume(&mut self) {
        self.byte_index += self.chars[self.char_index].len_utf8();
        self.char_index += 1;
    }

    // Consumes multiple chars, advancing the indices accordingly.
    fn consume_chars(&mut self, n: usize) {
        for _ in 0..n {
            self.consume();
        }
    }

    // Returns the char at the current index if any
    fn peek(&self) -> Option<char> {
        if self.is_done() {
            None
        } else {
            Some(self.chars[self.char_index])
        }
    }

    // Returns the char at the current index if any
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let index = self.char_index + n;
        if index >= self.chars.len() {
            None
        } else {
            Some(self.chars[index])
        }
    }

    fn count_while(&self, c: char) -> usize {
        let mut n = 1;
        while let Some(ahead) = self.peek_ahead(n) {
            if c == ahead {
                n += 1;
            } else {
                break;
            }
        }
        n
    }

    // Creates a substring of the input that starts at the current position, includes the initial
    // char and any subsequent char until the predicate is false.
    fn substr_while<P>(&self, predicate: P) -> Substr
    where
        P: Fn(char) -> bool,
    {
        let mut char_index = self.char_index;
        let mut to_index = self.byte_index + self.chars[char_index].len_utf8();
        char_index += 1;
        while let Some(c) = self.peek_ahead(char_index - self.char_index) {
            if predicate(c) {
                to_index += self.chars[char_index].len_utf8();
                char_index += 1;
            } else {
                break;
            }
        }
        self.input.substr(self.byte_index..to_index)
    }

    fn tokenize(mut self) -> Result<Tokens> {
        let mut errors: Errors = Default::default();
        while let Some(c) = self.peek() {
            if !self.add_known_first_char(c) && !self.add_digits(c) && !self.add_word(c) {
                errors.add(self.loc().to_wrap(Error::UnexpectedChar(c)));
                self.consume();
            }
        }
        errors.to_result(self.tokens)
    }

    fn add_known_first_char(&mut self, c: char) -> bool {
        match c {
            '\n' => self.add_token(Token::EndOfLine(NewLine::LF)),
            '\r' => self.add_token_if_next('\n', Token::EndOfLine(NewLine::CRLF)),
            ' ' => self.add_token(Token::Spaces(self.count_while(' '))),
            '\t' => self.add_token(Token::Tabs(self.count_while('\t'))),
            '+' => self.add_token(Token::Plus),
            '-' => self.add_token(Token::Minus),
            '*' => self.add_token(Token::Star),
            '/' => self.add_token(Token::Slash),
            '.' => self.add_token(Token::Dot),
            '>' => self.add_token_if_next_or_else('=', Token::GreaterOrEq, Token::Greater),
            '<' => self.add_token_if_next_or_else('=', Token::LessOrEq, Token::Less),
            '!' => self.add_token_if_next_or_else('=', Token::NotEquals, Token::Bang),
            '=' => self.add_token_if_next_or_else('=', Token::Equals, Token::Assignment),
            '&' => self.add_token_if_next_or_else('&', Token::LogicalAnd, Token::Ampersand),
            '|' => self.add_token_if_next_or_else('|', Token::LogicalOr, Token::Pipe),
            '(' => self.add_token(Token::Open(Enclosure::Parens)),
            ')' => self.add_token(Token::Close(Enclosure::Parens)),
            '{' => self.add_token(Token::Open(Enclosure::Curly)),
            '}' => self.add_token(Token::Close(Enclosure::Curly)),
            '[' => self.add_token(Token::Open(Enclosure::Square)),
            ']' => self.add_token(Token::Close(Enclosure::Square)),
            '_' => self.add_token(Token::Underscore),
            ':' => self.add_token_if_next_or_else(':', Token::DoubleColon, Token::Colon),
            _ => false,
        }
    }

    fn add_digits(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            self.add_token(Token::Digits(self.substr_while(|n| n.is_ascii_digit())))
        } else {
            false
        }
    }

    fn add_word(&mut self, c: char) -> bool {
        if c.is_ascii_alphabetic() {
            let word = self.substr_while(|n| n.is_ascii_alphanumeric());
            self.add_token(Token::Word(word))
        } else {
            false
        }
    }

    // Creates a token of the provided type consuming the needed chars.
    // Returns true to allow being the tail call of other add_ methods.
    fn add_token(&mut self, token: Token) -> bool {
        let chars = token.chars();
        self.tokens.push(self.loc().to_wrap(token));
        self.consume_chars(chars);
        true
    }

    fn add_token_if_next(&mut self, c: char, token: Token) -> bool {
        if let Some(next) = self.peek_ahead(1) {
            if next == c {
                return self.add_token(token);
            }
        }
        false
    }

    fn add_token_if_next_or_else(&mut self, c: char, token2: Token, token1: Token) -> bool {
        self.add_token_if_next(c, token2) || self.add_token(token1)
    }
}

#[cfg(test)]
mod tests;
