use super::{
    Enclosure, Keyword, Lexeme, LexemeKind, Lexemes, Loc, Result, Separator, Symbol, TSymbol,
    Token, L,
};
use crate::ArcStr;

fn assert_eq_noloc(actual: &Lexemes, expected: &Lexemes) {
    assert_eq!(actual.len(), expected.len());
    for (actual_lexeme, expected_lexeme) in actual.iter().zip(expected.iter()) {
        assert_eq!(
            actual_lexeme.separator, expected_lexeme.separator,
            "(Separator) Left=actual, Right=expected"
        );
        assert_eq!(
            actual_lexeme.kind, expected_lexeme.kind,
            "(Kind) Left=actual, Right=expected"
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

    fn token(mut self, separator: Separator, kind: LexemeKind) -> Self {
        self.lexemes.push(Lexeme::new(
            separator,
            Loc::input(self.input.clone(), 0).to_wrap(Token::Assignment),
            kind,
        ));
        self
    }

    fn integer(self, separator: Separator, n: i64) -> Self {
        self.token(separator, LexemeKind::Integer(n.into()))
    }

    fn open(self, separator: Separator, e: Enclosure) -> Self {
        self.token(separator, LexemeKind::Open(e))
    }

    fn close(self, separator: Separator, e: Enclosure) -> Self {
        self.token(separator, LexemeKind::Close(e))
    }

    fn lex(&self) -> Result<Lexemes> {
        super::lex(self.input.as_str())
    }

    fn id(self, separator: Separator, name: &str) -> Self {
        self.token(
            separator,
            LexemeKind::Symbol(Symbol::new(&Loc::None, name.into()).unwrap()),
        )
    }

    fn type_id(self, separator: Separator, name: &str) -> Self {
        self.token(
            separator,
            LexemeKind::TSymbol(TSymbol::new(&Loc::None, name.into()).unwrap()),
        )
    }

    fn keyword(self, separator: Separator, keyword: Keyword) -> Self {
        self.token(separator, LexemeKind::Keyword(keyword))
    }

    fn ok_without_pos(self) {
        match self.lex() {
            Ok(lexemes) => assert_eq_noloc(&lexemes, &self.lexemes),
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
    TestCase::new("1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn digits2() {
    TestCase::new("\t1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn digits3() {
    TestCase::new("\t 1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn add1() {
    TestCase::new("1234+456")
        .integer(Separator::NewLine, 1234)
        .token(Separator::Nothing, LexemeKind::Plus)
        .integer(Separator::Nothing, 456)
        .ok_without_pos();
}

#[test]
fn add2() {
    TestCase::new("  1234 +  456")
        .integer(Separator::NewLine, 1234)
        .token(Separator::Whitespace, LexemeKind::Plus)
        .integer(Separator::Whitespace, 456)
        .ok_without_pos();
}

#[test]
fn add3() {
    TestCase::new("  1234 +\n\t456")
        .integer(Separator::NewLine, 1234)
        .token(Separator::Whitespace, LexemeKind::Plus)
        .integer(Separator::NewLine, 456)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines1() {
    TestCase::new("\n\n \n1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines2() {
    TestCase::new("\n\n \n\t \n \t \n1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines3() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines4() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234\n\n \n 567\n\n")
        .integer(Separator::NewLine, 1234)
        .integer(Separator::NewLine, 567)
        .ok_without_pos();
}

#[test]
fn parens1() {
    TestCase::new("()")
        .open(Separator::NewLine, Enclosure::Parens)
        .close(Separator::Nothing, Enclosure::Parens)
        .ok_without_pos();
}

#[test]
fn parens2() {
    TestCase::new("((()))")
        .open(Separator::NewLine, Enclosure::Parens)
        .open(Separator::Nothing, Enclosure::Parens)
        .open(Separator::Nothing, Enclosure::Parens)
        .close(Separator::Nothing, Enclosure::Parens)
        .close(Separator::Nothing, Enclosure::Parens)
        .close(Separator::Nothing, Enclosure::Parens)
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
        .open(Separator::NewLine, Enclosure::Square)
        .open(Separator::Nothing, Enclosure::Curly)
        .open(Separator::Nothing, Enclosure::Parens)
        .integer(Separator::Nothing, 1234)
        .close(Separator::Nothing, Enclosure::Parens)
        .close(Separator::Nothing, Enclosure::Curly)
        .close(Separator::Nothing, Enclosure::Square)
        .ok_without_pos();
}

#[test]
fn assignment() {
    TestCase::new("let x = True")
        .keyword(Separator::NewLine, Keyword::Let)
        .id(Separator::Whitespace, "x")
        .token(Separator::Whitespace, LexemeKind::Assignment)
        .type_id(Separator::Whitespace, "True")
        .ok_without_pos();
}
