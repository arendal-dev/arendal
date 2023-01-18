use super::{Token, TokenKind, Tokens, Enclosure, NewLine, Pos};
use NewLine::*;

struct TestCase<'a> {
    input: &'a str,
    pos: Pos<'a>,
    tokens: Tokens<'a>,
}

impl<'a> TestCase<'a> {
    fn new(input: &'a str) -> TestCase<'a> {
        TestCase {
            input,
            pos: Pos::new(input),
            tokens: Default::default(),
        }
    }

    fn token(mut self, token_type: TokenKind<'a>, bytes: usize) -> Self {
        self.tokens.tokens.push(Token {
            pos: self.pos,
            kind: token_type,
        });
        self.pos.advance(bytes);
        self
    }

    fn newline(mut self, nl: NewLine) -> Self {
        self.token(TokenKind::EndOfLine(nl), nl.bytes())
    }

    fn single(self, token_type: TokenKind<'a>) -> Self {
        self.token(token_type, 1)
    }

    fn spaces(self, n: usize) -> Self {
        self.token(TokenKind::Spaces(n), n)
    }

    fn tabs(self, n: usize) -> Self {
        self.token(TokenKind::Tabs(n), n)
    }

    fn digits(self, digits: &'a str) -> Self {
        self.token(TokenKind::Digits(digits), digits.len())
    }

    fn word(self, word: &'a str) -> Self {
        self.token(TokenKind::Word(word), word.len())
    }

    fn ok(&self) {
        match super::tokenize(self.input) {
            Ok(tokens) => assert_eq!(tokens, self.tokens),
            Err(_) => panic!(),
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
        .single(TokenKind::Plus)
        .single(TokenKind::Minus)
        .single(TokenKind::Star)
        .single(TokenKind::Dot)
        .single(TokenKind::Slash)
        .single(TokenKind::Greater)
        .single(TokenKind::Less)
        .single(TokenKind::Bang)
        .single(TokenKind::Open(Enclosure::Parens))
        .single(TokenKind::Close(Enclosure::Parens))
        .single(TokenKind::Equal)
        .single(TokenKind::Open(Enclosure::Curly))
        .single(TokenKind::Close(Enclosure::Curly))
        .single(TokenKind::Open(Enclosure::Square))
        .single(TokenKind::Close(Enclosure::Square))
        .single(TokenKind::Underscore)
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
