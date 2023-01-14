use super::{Indentation, Pos, Token, TokenType, Tokens};
use arendal_ast::ToBigInt;

fn eq_types(left: &Tokens, right: &Tokens) -> bool {
    let n = left.len();
    if n == right.len() {
        for (left_token, right_token) in left.iter().zip(right.iter()) {
            if left_token.token_type != right_token.token_type {
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
    tokens: Tokens<'a>,
}

impl<'a> TestCase<'a> {
    fn new(input: &'a str) -> TestCase<'a> {
        TestCase {
            input,
            tokens: Vec::new(),
        }
    }

    fn token(mut self, token_type: TokenType<'a>) -> Self {
        self.tokens.push(Box::new(Token {
            pos: Pos::new(self.input),
            token_type,
        }));
        self
    }

    fn whitespace(self) -> Self {
        self.token(TokenType::Whitespace)
    }

    fn indentation(self, tabs: usize, spaces: usize) -> Self {
        self.token(TokenType::Indent(Indentation::new(tabs, spaces)))
    }

    fn integer(self, n: usize) -> Self {
        self.token(TokenType::Integer(n.to_bigint().unwrap()))
    }

    fn ok_without_pos(&self) {
        match super::tokenize(self.input) {
            Ok(tokens) => assert!(
                eq_types(&tokens, &self.tokens),
                "{:?}\n{:?}",
                &tokens,
                &self.tokens
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
        .token(TokenType::Plus)
        .integer(456)
        .ok_without_pos();
}

#[test]
fn sum2() {
    TestCase::new("  1234 +  456")
        .indentation(0, 2)
        .integer(1234)
        .whitespace()
        .token(TokenType::Plus)
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
        .token(TokenType::Plus)
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
