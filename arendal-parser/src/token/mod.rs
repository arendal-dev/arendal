use std::fmt;

use super::{
    chartoken, BigInt, CharToken, CharTokenType, CharTokens, Errors, Indentation, NewLine, Pos,
    Result,
};

pub fn tokenize(input: &str) -> Result<Tokens> {
    let pass1 = chartoken::tokenize(input)?;
    tokenize2(pass1)
}

fn tokenize2(input: CharTokens) -> Result<Tokens> {
    Tokenizer::new(input).tokenize()
}

pub type Tokens<'a> = Vec<Box<Token<'a>>>;

#[derive(Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub pos: Pos<'a>, // Starting position of the token
    pub token_type: TokenType<'a>,
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.token_type, self.pos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType<'a> {
    Indent(Indentation),
    Whitespace,
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
    NotEqual,
    Integer(BigInt),
    OpenParens,
    CloseParens,
    OpenCBracket,
    CloseCBracket,
    OpenSBracket,
    CloseSBracket,
    Underscore,
    Word(&'a str),
}

impl<'a> TokenType<'a> {
    fn single(t: &CharTokenType) -> Option<TokenType<'a>> {
        match t {
            CharTokenType::Plus => Some(TokenType::Plus),
            CharTokenType::Minus => Some(TokenType::Minus),
            CharTokenType::Star => Some(TokenType::Star),
            CharTokenType::Slash => Some(TokenType::Slash),
            CharTokenType::Dot => Some(TokenType::Dot),
            CharTokenType::Greater => Some(TokenType::Greater),
            CharTokenType::Less => Some(TokenType::Less),
            CharTokenType::Equal => Some(TokenType::Equal),
            CharTokenType::Underscore => Some(TokenType::Underscore),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    input: CharTokens<'a>,
    tokens: Tokens<'a>,
    errors: Errors<'a>,
    index: usize,         // Index of the current input token
    token_start: Pos<'a>, // Start of the current output token
}

impl<'a> Tokenizer<'a> {
    fn new(input: CharTokens<'a>) -> Tokenizer<'a> {
        Tokenizer {
            input,
            tokens: Vec::new(),
            errors: Default::default(),
            index: 0,
            token_start: Pos::new(""),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.index >= self.input.len()
    }

    // Consumes one token, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the token at the current index, if any
    fn peek(&self) -> Option<CharToken<'a>> {
        if self.is_done() {
            None
        } else {
            Some(self.input[self.index].clone())
        }
    }

    // Returns a clone of the token the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<CharToken<'a>> {
        let i = self.index + n;
        if i >= self.input.len() {
            None
        } else {
            Some(self.input[i].clone())
        }
    }

    fn tokenize(mut self) -> Result<'a, Tokens<'a>> {
        self.consume_indentation();
        while let Some(t) = self.peek() {
            self.token_start = t.pos;
            if self.consume_whitespace(true) {
                continue;
            }
            if let Some(tt) = TokenType::single(&t.token_type) {
                self.consume();
                self.add_token(tt);
                continue;
            }
            match t.token_type {
                CharTokenType::EndOfLine(_) => {
                    self.consume();
                    self.consume_indentation();
                }
                CharTokenType::Bang => self.consume_bang(),
                CharTokenType::Digits(s) => self.consume_digits(s),
                _ => self.errors.add(crate::parsing_error(t.pos)),
            }
        }
        self.errors.to_result(self.tokens)
    }

    fn add_token(&mut self, token_type: TokenType<'a>) {
        self.tokens.push(Box::new(Token {
            pos: self.token_start,
            token_type,
        }));
    }

    fn consume_tabs(&mut self) -> usize {
        let mut tabs = 0;
        while let Some(CharToken {
            token_type: CharTokenType::Tabs(n),
            ..
        }) = self.peek()
        {
            tabs += n;
            self.consume();
        }
        tabs
    }

    fn consume_spaces(&mut self) -> usize {
        let mut spaces = 0;
        while let Some(CharToken {
            token_type: CharTokenType::Spaces(n),
            ..
        }) = self.peek()
        {
            spaces += n;
            self.consume();
        }
        spaces
    }

    fn consume_whitespace(&mut self, add_token: bool) -> bool {
        let mut found = false;
        while let Some(t) = self.peek() {
            if t.is_whitespace() {
                found = true;
                self.consume();
            } else {
                break;
            }
        }
        if found && add_token {
            self.add_token(TokenType::Whitespace);
        }
        found
    }

    fn skip_empty_lines(&mut self) {
        // Skip empty lines
        let mut look_ahead: usize = 0;
        let mut remove: usize = 0;
        let mut line_index: usize = 0;
        while let Some(token) = self.peek_ahead(look_ahead) {
            if token.is_whitespace() {
                look_ahead += 1;
                line_index += 1;
                continue;
            }
            if let CharTokenType::EndOfLine(_) = token.token_type {
                look_ahead += 1;
                remove += line_index + 1; // the EOL itself
                line_index = 0;
                continue;
            }
            break; // any other token
        }
        self.index += remove;
    }

    fn consume_indentation(&mut self) {
        self.skip_empty_lines();
        if let Some(token) = self.peek() {
            self.token_start = token.pos;
            let tabs = self.consume_tabs();
            let spaces = self.consume_spaces();
            self.add_token(TokenType::Indent(Indentation::new(tabs, spaces)));
            if let Some(t) = self.peek() {
                if t.is_whitespace() {
                    self.add_indentation_error(&t);
                    self.consume_whitespace(false);
                }
            }
        }
    }

    fn consume_bang(&mut self) {
        let t = if let Some(CharTokenType::Equal) = self.peek_ahead(1).map(|t| t.token_type) {
            self.consume();
            TokenType::NotEqual
        } else {
            TokenType::Bang
        };
        self.add_token(t);
        self.consume();
    }

    fn consume_digits(&mut self, digits: &'a str) {
        self.consume();
        self.add_token(TokenType::Integer(digits.parse().unwrap()));
    }

    fn add_indentation_error(&mut self, token: &CharToken<'a>) {
        self.errors.add(crate::indentation_error(token.pos))
    }
}

#[cfg(test)]
mod tests;
