use super::{ArcStr, Enclosure, Loc, NewLine, Token, Tokens};
use NewLine::*;

struct TestCase {
    input: ArcStr,
    index: usize,
    tokens: Tokens,
}

impl TestCase {
    fn new(input: &str) -> TestCase {
        let arcstr = ArcStr::from(input);
        TestCase {
            input: arcstr.clone(),
            index: 0,
            tokens: Default::default(),
        }
    }

    fn token(mut self, token: Token) -> Self {
        let bytes = token.bytes();
        self.tokens
            .push(Loc::input(self.input.clone(), self.index).to_wrap(token));
        self.index += bytes;
        self
    }

    fn newline(self, nl: NewLine) -> Self {
        self.token(Token::EndOfLine(nl))
    }

    fn spaces(self, n: usize) -> Self {
        self.token(Token::Spaces(n))
    }

    fn tabs(self, n: usize) -> Self {
        self.token(Token::Tabs(n))
    }

    fn digits(self, digits: &str) -> Self {
        self.token(Token::Digits(ArcStr::from(digits).substr(0..)))
    }

    fn word(self, word: &str) -> Self {
        self.token(Token::Word(ArcStr::from(word).substr(0..)))
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
        .token(Token::Plus)
        .token(Token::Minus)
        .token(Token::Star)
        .token(Token::Dot)
        .token(Token::Slash)
        .token(Token::Greater)
        .token(Token::Less)
        .token(Token::Bang)
        .token(Token::Open(Enclosure::Parens))
        .token(Token::Close(Enclosure::Parens))
        .token(Token::Assignment)
        .token(Token::Open(Enclosure::Curly))
        .token(Token::Close(Enclosure::Curly))
        .token(Token::Open(Enclosure::Square))
        .token(Token::Close(Enclosure::Square))
        .token(Token::Underscore)
        .ok();
}

#[test]
fn bang() {
    TestCase::new("!a!=b")
        .token(Token::Bang)
        .word("a")
        .token(Token::NotEquals)
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

#[test]
fn sum() {
    TestCase::new("1 + 3")
        .digits("1")
        .spaces(1)
        .token(Token::Plus)
        .spaces(1)
        .digits("3")
        .ok();
}
