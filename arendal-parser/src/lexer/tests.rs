use super::{
    ArcStr, Enclosure, Indentation, Lexeme, LexemeKind, LexemeRef, Lexemes, Pos, Result, Token,
    TokenKind,
};

fn eq_kinds(left: &Lexemes, right: &Lexemes) -> bool {
    let n = left.lexemes.len();
    if n == right.lexemes.len() {
        for (left_token, right_token) in left.lexemes.iter().zip(right.lexemes.iter()) {
            if left_token.kind() != right_token.kind() {
                return false;
            }
        }
        true
    } else {
        false
    }
}

struct TestCase {
    input: ArcStr,
    lexemes: Lexemes,
}

impl TestCase {
    fn new(input: &str) -> TestCase {
        TestCase {
            input: ArcStr::from(input),
            lexemes: Default::default(),
        }
    }

    // New test case with no identation
    fn new0(input: &str) -> TestCase {
        Self::new(input).indentation(0, 0)
    }

    fn token(mut self, kind: LexemeKind) -> Self {
        self.lexemes.lexemes.push(LexemeRef::new(Lexeme {
            token: Token {
                pos: Pos::new(self.input.clone()),
                kind: TokenKind::Equal,
            },
            kind,
        }));
        self
    }

    fn indentation(self, tabs: usize, spaces: usize) -> Self {
        self.token(LexemeKind::Indent(Indentation::new(tabs, spaces)))
    }

    fn integer(self, n: i64) -> Self {
        self.token(LexemeKind::Integer(n.into()))
    }

    fn open(self, e: Enclosure) -> Self {
        self.token(LexemeKind::Open(e))
    }

    fn close(self, e: Enclosure) -> Self {
        self.token(LexemeKind::Close(e))
    }

    fn lex(&self) -> Result<Lexemes> {
        super::lex(self.input.as_str())
    }

    fn ok_without_pos(self) {
        match self.lex() {
            Ok(tokens) => assert!(
                eq_kinds(&tokens, &self.lexemes),
                "{:?}\n{:?}",
                &tokens,
                &self.lexemes
            ),
            Err(_) => panic!(),
        }
    }

    fn err(self) {
        match self.lex() {
            Ok(_) => panic!(),
            Err(_) => (),
        }
    }
}

#[test]
fn empty() {
    TestCase::new("").ok_without_pos();
}

#[test]
fn digits1() {
    TestCase::new0("1234").integer(1234).ok_without_pos();
}

#[test]
fn digits2() {
    TestCase::new("\t1234")
        .indentation(1, 0)
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn digits3() {
    TestCase::new("\t 1234")
        .indentation(1, 1)
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn sum1() {
    TestCase::new0("1234+456")
        .integer(1234)
        .token(LexemeKind::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn sum2() {
    TestCase::new("  1234 +  456")
        .indentation(0, 2)
        .integer(1234)
        .token(LexemeKind::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn sum3() {
    TestCase::new("  1234 +\n\t456")
        .indentation(0, 2)
        .integer(1234)
        .token(LexemeKind::Plus)
        .indentation(1, 0)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines1() {
    TestCase::new("\n\n \n1234")
        .indentation(0, 0)
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines2() {
    TestCase::new("\n\n \n\t \n \t \n1234")
        .indentation(0, 0)
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines3() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234")
        .indentation(1, 1)
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines4() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234\n\n \n 567\n\n")
        .indentation(1, 1)
        .integer(1234)
        .indentation(0, 1)
        .integer(567)
        .ok_without_pos();
}

#[test]
fn parens1() {
    TestCase::new0("()")
        .open(Enclosure::Parens)
        .close(Enclosure::Parens)
        .ok_without_pos();
}

#[test]
fn parens2() {
    TestCase::new0("((()))")
        .open(Enclosure::Parens)
        .open(Enclosure::Parens)
        .open(Enclosure::Parens)
        .close(Enclosure::Parens)
        .close(Enclosure::Parens)
        .close(Enclosure::Parens)
        .ok_without_pos();
}

#[test]
fn parens_err1() {
    TestCase::new0(")").err();
}

#[test]
fn parens_err2() {
    TestCase::new0("(()))").err();
}

#[test]
fn enclosures_mixed_ok() {
    TestCase::new0("[{(1234)}]")
        .open(Enclosure::Square)
        .open(Enclosure::Curly)
        .open(Enclosure::Parens)
        .integer(1234)
        .close(Enclosure::Parens)
        .close(Enclosure::Curly)
        .close(Enclosure::Square)
        .ok_without_pos();
}
