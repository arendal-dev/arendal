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
