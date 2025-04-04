use std::rc::Rc;

use super::{
    Enclosure, Keyword, Level, Lexeme, LexemeData, Lexemes, Position, Result, Separator, Symbol, TSymbol};
use arcstr::ArcStr;
use ast::input::StringInput;

fn assert_eq_noloc(actual: &Vec<Lexeme>, expected: &Vec<Lexeme>) {
    assert_eq!(actual.len(), expected.len());
    for (actual_lexeme, expected_lexeme) in actual.iter().zip(expected.iter()) {
        assert_eq!(
            actual_lexeme.separator, expected_lexeme.separator,
            "(Separator) Left=actual, Right=expected"
        );
        assert_eq!(
            actual_lexeme.data, expected_lexeme.data,
            "(Kind) Left=actual, Right=expected"
        );
    }
}

struct Parent {
    separator: Separator,
    enclosure: Enclosure,
    parent: TestCase,
}


struct TestCase {
    parent: Option<Box<Parent>>,
    input: ArcStr,
    lexemes: Vec<Lexeme>,
}

fn test(input: &str) -> TestCase {
    TestCase {
        parent: None,
        input: ArcStr::from(input),
        lexemes: Vec::default(),
    }
}

impl TestCase {

    fn token(mut self, separator: Separator, data: LexemeData) -> Self {
        self.lexemes.push(Lexeme {
            position: Position::NoPosition,
            separator,
            data
        });
        self
    }

    fn integer(self, separator: Separator, n: i64) -> Self {
        self.token(separator, LexemeData::Integer(n.into()))
    }

    fn level(self, separator: Separator, enclosure: Enclosure) -> Self {
        let input = self.input.clone();
        let parent = Parent {
            separator,
            enclosure,
            parent: self
        };
        TestCase {
            parent: Some(Box::new(parent)),
            input,
            lexemes: Vec::default(),
        }
    }

    fn close(self) -> Self {
        match self.parent {
            None => panic!("Missing parent enclosure"),
            Some(mut parent) => {
                let lexemes = Lexemes { lexemes: Rc::new(self.lexemes) };
                let level = Level {
                    enclosure: parent.enclosure,
                    lexemes
                };
                parent.parent.lexemes.push(Lexeme { position: Position::NoPosition, separator: parent.separator, data: LexemeData::Level(level) });
                parent.parent
            }
        }
    }

    fn symbol(self, separator: Separator, name: &str) -> Self {
        self.token(separator, LexemeData::Symbol(Symbol::new(name).unwrap()))
    }

    fn tsymbol(self, separator: Separator, name: &str) -> Self {
        self.token(separator, LexemeData::TSymbol(TSymbol::new(name).unwrap()))
    }

    fn keyword(self, separator: Separator, keyword: Keyword) -> Self {
        self.token(separator, LexemeData::Keyword(keyword))
    }

    fn lex(&self) -> Result<Lexemes> {
        assert!(self.parent.is_none());
        super::lex(StringInput::from_arcstr(self.input.clone()))
    }

    fn ok_without_pos(self) {
        match self.parent {
            None => match self.lex() {
                Ok(lexemes) => assert_eq_noloc(&lexemes.0.lexemes, &self.lexemes),
                Err(_) => panic!(),
            },
            _ => self.close().ok_without_pos(),
        }
    }

    fn err(self) {
        match self.parent {
            None => match self.lex() {
                Ok(_) => panic!(),
                Err(_) => (),
            },
            _ => self.close().err(),
        }
        
    }
}

#[test]
fn empty() {
    test("").ok_without_pos();
}

#[test]
fn digits1() {
    test("1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn digits2() {
    test("\t1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn digits3() {
    test("\t 1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn add1() {
    test("1234+456")
        .integer(Separator::NewLine, 1234)
        .token(Separator::Nothing, LexemeData::Plus)
        .integer(Separator::Nothing, 456)
        .ok_without_pos();
}

#[test]
fn add2() {
    test("  1234 +  456")
        .integer(Separator::NewLine, 1234)
        .token(Separator::Whitespace, LexemeData::Plus)
        .integer(Separator::Whitespace, 456)
        .ok_without_pos();
}

#[test]
fn add3() {
    test("  1234 +\n\t456")
        .integer(Separator::NewLine, 1234)
        .token(Separator::Whitespace, LexemeData::Plus)
        .integer(Separator::NewLine, 456)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines1() {
    test("\n\n \n1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines2() {
    test("\n\n \n\t \n \t \n1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines3() {
    test("\n\n \n\t \n \t \n\t 1234")
        .integer(Separator::NewLine, 1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines4() {
    test("\n\n \n\t \n \t \n\t 1234\n\n \n 567\n\n")
        .integer(Separator::NewLine, 1234)
        .integer(Separator::NewLine, 567)
        .ok_without_pos();
}

#[test]
fn parens1() {
    test("()")
        .level(Separator::NewLine, Enclosure::Parens)
        .ok_without_pos();
}

#[test]
fn parens2() {
    test("((()))")
        .level(Separator::NewLine, Enclosure::Parens)
        .level(Separator::Nothing, Enclosure::Parens)
        .level(Separator::Nothing, Enclosure::Parens)
        .ok_without_pos();
}

#[test]
fn parens_err1() {
    test(")").err();
}

#[test]
fn parens_err2() {
    test("(()))").err();
}

#[test]
fn enclosures_mixed_ok() {
    test("[{(1234)}]")
        .level(Separator::NewLine, Enclosure::Square)
        .level(Separator::Nothing, Enclosure::Curly)
        .level(Separator::Nothing, Enclosure::Parens)
        .integer(Separator::Nothing, 1234)
        .ok_without_pos();
}

#[test]
fn assignment() {
    test("let x = True")
        .keyword(Separator::NewLine, Keyword::Let)
        .symbol(Separator::Whitespace, "x")
        .token(Separator::Whitespace, LexemeData::Assignment)
        .tsymbol(Separator::Whitespace, "True")
        .ok_without_pos();
}

#[test]
fn path() {
    test("a::b::C")
        .symbol(Separator::NewLine, "a")
        .token(Separator::Nothing, LexemeData::PathSeparator)
        .symbol(Separator::Nothing, "b")
        .token(Separator::Nothing, LexemeData::PathSeparator)
        .tsymbol(Separator::Nothing, "C")
        .ok_without_pos();
}
