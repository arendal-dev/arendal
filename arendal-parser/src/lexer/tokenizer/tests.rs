use super::{Enclosure, TokenKind, Tokenizer};
use ast::input::StringInput;

struct TestCase {
    input: StringInput,
    tokenizer: Tokenizer,
}

fn test(input: &str) -> TestCase {
    let input = StringInput::from_str(input);
    let tokenizer = Tokenizer::new(input.clone());
    TestCase { input, tokenizer }
}

impl TestCase {
    fn token(mut self, content: &str, kind: TokenKind) -> Self {
        for c in content.chars() {
            self.tokenizer.advance(c);
        }
        self.tokenizer.add_token(kind);
        self
    }

    fn lf(self) -> Self {
        self.token("\n", TokenKind::NewLine)
    }

    fn crlf(self) -> Self {
        self.token("\r\n", TokenKind::NewLine)
    }

    fn spaces(mut self, n: usize) -> Self {
        for _ in 0..n {
            self.tokenizer.advance(' ');
        }
        self.tokenizer.add_token(TokenKind::Spaces);
        self
    }

    fn tabs(mut self, n: usize) -> Self {
        for _ in 0..n {
            self.tokenizer.advance('\t');
        }
        self.tokenizer.add_token(TokenKind::Tabs);
        self
    }

    fn digits(self, digits: &str) -> Self {
        self.token(digits, TokenKind::Digits)
    }

    fn word(self, word: &str) -> Self {
        self.token(word, TokenKind::Word)
    }

    fn ok(self) {
        assert_eq!(
            *super::tokenize(self.input.clone()).tokens,
            self.tokenizer.tokens
        )
    }
}

#[test]
fn empty() {
    test("").ok();
}

#[test]
fn spaces() {
    test("   ").spaces(3).ok();
}

#[test]
fn tabs() {
    test("\t\t\t").tabs(3).ok();
}

#[test]
fn lf() {
    test("\n").lf().ok();
}

#[test]
fn crlf() {
    test("\r\n").crlf().ok();
}

#[test]
fn singles() {
    test("+-*./><!()={}[]_:")
        .token("+", TokenKind::Plus)
        .token("-", TokenKind::Minus)
        .token("+", TokenKind::Star)
        .token(".", TokenKind::Dot)
        .token("/", TokenKind::Slash)
        .token(">", TokenKind::Greater)
        .token("<", TokenKind::Less)
        .token("!", TokenKind::Bang)
        .token("(", TokenKind::Open(Enclosure::Parens))
        .token(")", TokenKind::Close(Enclosure::Parens))
        .token("=", TokenKind::Assignment)
        .token("{", TokenKind::Open(Enclosure::Curly))
        .token("}", TokenKind::Close(Enclosure::Curly))
        .token("[", TokenKind::Open(Enclosure::Square))
        .token("]", TokenKind::Close(Enclosure::Square))
        .token("_", TokenKind::Underscore)
        .token(":", TokenKind::Colon)
        .ok();
}

#[test]
fn bang() {
    test("!a!=b")
        .token("!", TokenKind::Bang)
        .word("a")
        .token("!=", TokenKind::NotEquals)
        .word("b")
        .ok();
}

#[test]
fn digits1() {
    test("1234").digits("1234").ok();
}

#[test]
fn digits2() {
    test("12 34").digits("12").spaces(1).digits("34").ok();
}

#[test]
fn word1() {
    test("abc").word("abc").ok();
}

#[test]
fn word2() {
    test("abc4e").word("abc4e").ok();
}

#[test]
fn word3() {
    test("4bc5e").digits("4").word("bc5e").ok();
}

#[test]
fn harness() {
    test("   \n\t").spaces(3).lf().tabs(1).ok();
}

#[test]
fn sum() {
    test("1 + 3")
        .digits("1")
        .spaces(1)
        .token("+", TokenKind::Plus)
        .spaces(1)
        .digits("3")
        .ok();
}

#[test]
fn path() {
    test("a::b::C")
        .word("a")
        .token("::", TokenKind::DoubleColon)
        .word("b")
        .token("::", TokenKind::DoubleColon)
        .word("C")
        .ok();
}
