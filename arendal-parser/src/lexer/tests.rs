use core::error::Loc;

use super::{
    Enclosure, Inner, Keyword, Lexeme, LexemeKind, Lexemes, Result, Symbol, TSymbol, Token,
    TokenKind,
};
use core::ArcStr;

fn assert_eq_kinds(actual: &Lexemes, expected: &Lexemes) {
    assert_eq!(actual.lexemes.len(), expected.lexemes.len());
    for (actual_lexeme, expected_lexeme) in actual.lexemes.iter().zip(expected.lexemes.iter()) {
        assert_eq!(
            actual_lexeme.kind(),
            expected_lexeme.kind(),
            "Left=actual, Right=expected"
        );
    }
}

struct TestCase {
    input: ArcStr,
    lexemes: Vec<Lexeme>,
}

impl TestCase {
    fn new(input: &str) -> TestCase {
        TestCase {
            input: ArcStr::from(input),
            lexemes: Default::default(),
        }
    }

    fn token(mut self, kind: LexemeKind) -> Self {
        self.lexemes.push(Lexeme::new(Inner {
            token: Token {
                loc: Loc::input(self.input.clone(), 0),
                kind: TokenKind::Assignment,
            },
            kind,
        }));
        self
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

    fn id(self, name: &str) -> Self {
        self.token(LexemeKind::Id(
            Symbol::new(Loc::none(), name.into()).unwrap(),
        ))
    }

    fn type_id(self, name: &str) -> Self {
        self.token(LexemeKind::TypeId(
            TSymbol::new(Loc::none(), name.into()).unwrap(),
        ))
    }

    fn keyword(self, keyword: Keyword) -> Self {
        self.token(LexemeKind::Keyword(keyword))
    }

    fn ok_without_pos(mut self) {
        match self.lex() {
            Ok(lexemes) => assert_eq_kinds(&lexemes, &Lexemes::new(&mut self.lexemes)),
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
    TestCase::new("1234").integer(1234).ok_without_pos();
}

#[test]
fn digits2() {
    TestCase::new("\t1234").integer(1234).ok_without_pos();
}

#[test]
fn digits3() {
    TestCase::new("\t 1234").integer(1234).ok_without_pos();
}

#[test]
fn add1() {
    TestCase::new("1234+456")
        .integer(1234)
        .token(LexemeKind::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn add2() {
    TestCase::new("  1234 +  456")
        .integer(1234)
        .token(LexemeKind::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn add3() {
    TestCase::new("  1234 +\n\t456")
        .integer(1234)
        .token(LexemeKind::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines1() {
    TestCase::new("\n\n \n1234").integer(1234).ok_without_pos();
}

#[test]
fn remove_empty_lines2() {
    TestCase::new("\n\n \n\t \n \t \n1234")
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines3() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234")
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines4() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234\n\n \n 567\n\n")
        .integer(1234)
        .integer(567)
        .ok_without_pos();
}

#[test]
fn parens1() {
    TestCase::new("()")
        .open(Enclosure::Parens)
        .close(Enclosure::Parens)
        .ok_without_pos();
}

#[test]
fn parens2() {
    TestCase::new("((()))")
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
    TestCase::new(")").err();
}

#[test]
fn parens_err2() {
    TestCase::new("(()))").err();
}

#[test]
fn enclosures_mixed_ok() {
    TestCase::new("[{(1234)}]")
        .open(Enclosure::Square)
        .open(Enclosure::Curly)
        .open(Enclosure::Parens)
        .integer(1234)
        .close(Enclosure::Parens)
        .close(Enclosure::Curly)
        .close(Enclosure::Square)
        .ok_without_pos();
}

#[test]
fn assignment() {
    TestCase::new("val x = True")
        .keyword(Keyword::Val)
        .id("x")
        .token(LexemeKind::Assignment)
        .type_id("True")
        .ok_without_pos();
}
