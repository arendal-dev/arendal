use arcstr::ArcStr;
use super::{Enclosure, NewLine, TokenData, Tokenizer};
use ast::input::StringInput;
use NewLine::*;

struct TestCase {
    input: StringInput,
    tokenizer: Tokenizer,
}

impl TestCase {
    fn new(input: &str) -> TestCase {
        let input = StringInput::from_str(input);
        let tokenizer = Tokenizer::new(input.clone());
        TestCase {
            input,
            tokenizer,
        }
    }

    fn token(mut self, data: TokenData) -> Self {
        self.tokenizer.add_token(data);
        self
    }

    fn newline(self, nl: NewLine) -> Self {
        self.token(TokenData::EndOfLine(nl))
    }

    fn spaces(self, n: usize) -> Self {
        self.token(TokenData::Spaces(n))
    }

    fn tabs(self, n: usize) -> Self {
        self.token(TokenData::Tabs(n))
    }

    fn digits(self, digits: &str) -> Self {
        self.token(TokenData::Digits(ArcStr::from(digits).substr(0..)))
    }

    fn word(self, word: &str) -> Self {
        self.token(TokenData::Word(ArcStr::from(word).substr(0..)))
    }

    fn ok(&self) {
        match super::tokenize(self.input.clone()) {
            Ok(output) => assert_eq!(*output.0.tokens, self.tokenizer.tokens),
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
    TestCase::new("+-*./><!()={}[]_:")
        .token(TokenData::Plus)
        .token(TokenData::Minus)
        .token(TokenData::Star)
        .token(TokenData::Dot)
        .token(TokenData::Slash)
        .token(TokenData::Greater)
        .token(TokenData::Less)
        .token(TokenData::Bang)
        .token(TokenData::Open(Enclosure::Parens))
        .token(TokenData::Close(Enclosure::Parens))
        .token(TokenData::Assignment)
        .token(TokenData::Open(Enclosure::Curly))
        .token(TokenData::Close(Enclosure::Curly))
        .token(TokenData::Open(Enclosure::Square))
        .token(TokenData::Close(Enclosure::Square))
        .token(TokenData::Underscore)
        .token(TokenData::Colon)
        .ok();
}

#[test]
fn bang0() {
    TestCase::new("!a")
        .token(TokenData::Bang)
        .word("a")
        .ok();
}

#[test]
fn bang() {
    TestCase::new("!a!=b")
        .token(TokenData::Bang)
        .word("a")
        .token(TokenData::NotEquals)
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
        .token(TokenData::Plus)
        .spaces(1)
        .digits("3")
        .ok();
}

#[test]
fn path() {
    TestCase::new("a::b::C")
        .word("a")
        .token(TokenData::DoubleColon)
        .word("b")
        .token(TokenData::DoubleColon)
        .word("C")
        .ok();
}
