use super::{
    Enclosure, Indentation, Lexeme, LexemeKind, LexemeRef, Lexemes, Lines, Pos, Result, Token,
    TokenKind,
};
use crate::ArcStr;

fn assert_eq_lines(actual: &Lines, expected: &Lines) {
    assert_eq!(actual.lines.len(), actual.lines.len());
    for (actual_line, expected_line) in actual.lines.iter().zip(expected.lines.iter()) {
        assert_eq!(
            actual_line.indentation, expected_line.indentation,
            "Left is actual"
        );
        assert_eq_kinds(&actual_line.lexemes, &expected_line.lexemes);
    }
}

fn assert_eq_kinds(actual: &Lexemes, expected: &Lexemes) {
    assert_eq!(actual.lexemes.len(), expected.lexemes.len());
    for (actual_lexeme, expected_lexeme) in actual.lexemes.iter().zip(expected.lexemes.iter()) {
        assert_eq!(actual_lexeme.kind(), expected_lexeme.kind());
    }
}

struct TestCase {
    input: ArcStr,
    lexemes: Vec<LexemeRef>,
    indentation: Indentation,
    lines: Lines,
}

impl TestCase {
    fn new(input: &str, tabs: usize, spaces: usize) -> TestCase {
        TestCase {
            input: ArcStr::from(input),
            lexemes: Default::default(),
            indentation: Indentation::new(tabs, spaces),
            lines: Default::default(),
        }
    }

    // New test case with no identation in the first line
    fn new0(input: &str) -> TestCase {
        Self::new(input, 0, 0)
    }

    fn end_line(&mut self) {
        self.lines.add(self.indentation, &mut self.lexemes);
    }

    // Starts a new line with the provided indentation
    // The previous line must have some lexeme.
    fn indentation(mut self, tabs: usize, spaces: usize) -> Self {
        assert!(!self.lexemes.is_empty(), "No lexemes in the current line");
        self.end_line();
        self.indentation = Indentation::new(tabs, spaces);
        self
    }

    fn token(mut self, kind: LexemeKind) -> Self {
        self.lexemes.push(LexemeRef::new(Lexeme {
            token: Token {
                pos: Pos::new(self.input.clone()),
                kind: TokenKind::Equal,
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

    fn lex(&self) -> Result<Lines> {
        super::lex(self.input.as_str())
    }

    fn ok_without_pos(mut self) {
        if !self.lexemes.is_empty() {
            self.end_line();
        }
        match self.lex() {
            Ok(lines) => assert_eq_lines(&lines, &self.lines),
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
    TestCase::new0("").ok_without_pos();
}

#[test]
fn digits1() {
    TestCase::new0("1234").integer(1234).ok_without_pos();
}

#[test]
fn digits2() {
    TestCase::new("\t1234", 1, 0).integer(1234).ok_without_pos();
}

#[test]
fn digits3() {
    TestCase::new("\t 1234", 1, 1)
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
    TestCase::new("  1234 +  456", 0, 2)
        .integer(1234)
        .token(LexemeKind::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn sum3() {
    TestCase::new("  1234 +\n\t456", 0, 2)
        .integer(1234)
        .token(LexemeKind::Plus)
        .indentation(1, 0)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines1() {
    TestCase::new0("\n\n \n1234").integer(1234).ok_without_pos();
}

#[test]
fn remove_empty_lines2() {
    TestCase::new0("\n\n \n\t \n \t \n1234")
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines3() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234", 1, 1)
        .integer(1234)
        .ok_without_pos();
}

#[test]
fn remove_empty_lines4() {
    TestCase::new("\n\n \n\t \n \t \n\t 1234\n\n \n 567\n\n", 1, 1)
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
