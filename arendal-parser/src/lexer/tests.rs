use std::rc::Rc;
use super::{Indentation, Lexeme, LexemeKind, Lexemes, Pos};
use arendal_ast::ToBigInt;

fn eq_kinds(left: &Lexemes, right: &Lexemes) -> bool {
    let n = left.lexemes.len();
    if n == right.lexemes.len() {
        for (left_token, right_token) in left.lexemes.iter().zip(right.lexemes.iter()) {
            if left_token.kind != right_token.kind {
                return false;
            }
        }
        true
    } else {
        false
    }
}

struct TestCase<'a> {
    input: &'a str,
    lexemes: Lexemes<'a>,
}

impl<'a> TestCase<'a> {
    fn new(input: &'a str) -> TestCase<'a> {
        TestCase {
            input,
            lexemes: Default::default(),
        }
    }

    fn token(mut self, kind: LexemeKind<'a>) -> Self {
        self.lexemes.lexemes.push(Rc::new(Lexeme {
            pos: Pos::new(self.input),
            kind,
        }));
        self
    }

    fn whitespace(self) -> Self {
        self.token(LexemeKind::Whitespace)
    }

    fn indentation(self, tabs: usize, spaces: usize) -> Self {
        self.token(LexemeKind::Indent(Indentation::new(tabs, spaces)))
    }

    fn integer(self, n: usize) -> Self {
        self.token(LexemeKind::Integer(n.to_bigint().unwrap()))
    }

    fn ok_without_pos(&self) {
        match super::lex(self.input) {
            Ok(tokens) => assert!(
                eq_kinds(&tokens, &self.lexemes),
                "{:?}\n{:?}",
                &tokens,
                &self.lexemes
            ),
            Err(_) => panic!(),
        }
    }
}

#[test]
fn empty() {
    TestCase::new("").ok_without_pos();
}

#[test]
fn digits1() {
    TestCase::new("1234")
        .indentation(0, 0)
        .integer(1234)
        .ok_without_pos();
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
    TestCase::new("1234+456")
        .indentation(0, 0)
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
        .whitespace()
        .token(LexemeKind::Plus)
        .whitespace()
        .integer(456)
        .ok_without_pos();
}

#[test]
fn sum3() {
    TestCase::new("  1234 +\n\t456")
        .indentation(0, 2)
        .integer(1234)
        .whitespace()
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
