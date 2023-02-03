use super::{ArcStr, Enclosure, NewLine, Pos, Token, TokenKind, Tokens};
use NewLine::*;

struct TestCase {
    input: ArcStr,
    pos: Pos,
    tokens: Tokens,
}

impl TestCase {
    fn new(input: &str) -> TestCase {
        let arcstr = ArcStr::from(input);
        TestCase {
            input: arcstr.clone(),
            pos: Pos::new(arcstr),
            tokens: Default::default(),
        }
    }

    fn token(mut self, kind: TokenKind) -> Self {
        let bytes = kind.bytes();
        self.tokens.tokens.push(Token {
            pos: self.pos.clone(),
            kind,
        });
        self.pos.advance(bytes);
        self
    }

    fn newline(self, nl: NewLine) -> Self {
        self.token(TokenKind::EndOfLine(nl))
    }

    fn spaces(self, n: usize) -> Self {
        self.token(TokenKind::Spaces(n))
    }

    fn tabs(self, n: usize) -> Self {
        self.token(TokenKind::Tabs(n))
    }

    fn digits(self, digits: &str) -> Self {
        self.token(TokenKind::Digits(ArcStr::from(digits).substr(0..)))
    }

    fn word(self, word: &str) -> Self {
        self.token(TokenKind::Word(ArcStr::from(word).substr(0..)))
    }

    fn ok(&self) {
        match super::tokenize(self.input.as_str()) {
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
        .token(TokenKind::Plus)
        .token(TokenKind::Minus)
        .token(TokenKind::Star)
        .token(TokenKind::Dot)
        .token(TokenKind::Slash)
        .token(TokenKind::Greater)
        .token(TokenKind::Less)
        .token(TokenKind::Bang)
        .token(TokenKind::Open(Enclosure::Parens))
        .token(TokenKind::Close(Enclosure::Parens))
        .token(TokenKind::Equal)
        .token(TokenKind::Open(Enclosure::Curly))
        .token(TokenKind::Close(Enclosure::Curly))
        .token(TokenKind::Open(Enclosure::Square))
        .token(TokenKind::Close(Enclosure::Square))
        .token(TokenKind::Underscore)
        .ok();
}

#[test]
fn bang() {
    TestCase::new("!a!=b")
        .token(TokenKind::Bang)
        .word("a")
        .token(TokenKind::NotEquals)
        .word("b")
        .ok();
}

#[test]
fn digits1() {
    TestCase::new("1234").digits("1234").ok();
}

#[test]
fn digits2() {
    TestCase::new("12 34")
        .digits("12")
        .spaces(1)
        .digits("34")
        .ok();
}

#[test]
fn word1() {
    TestCase::new("abc").word("abc").ok();
}

#[test]
fn word2() {
    TestCase::new("abc4e").word("abc4e").ok();
}

#[test]
fn word3() {
    TestCase::new("4bc5e").digits("4").word("bc5e").ok();
}

#[test]
fn harness() {
    TestCase::new("   \n\t").spaces(3).newline(LF).tabs(1).ok();
}
