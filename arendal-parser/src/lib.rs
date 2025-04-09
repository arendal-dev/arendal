mod lexer;

use ast::{problem::Problems, stmt::Expression};
use lexer::{Lexeme, LexemeData, Lexemes};

struct Parser {
    lexemes: Lexemes,
    index: usize,
    problems: Problems,
}

type EResult = std::result::Result<Expression, ()>;

impl Parser {
    fn peek(&self) -> Option<&Lexeme> {
        self.lexemes.get(self.index)
    }

    fn rule_primary(&self) -> EResult {
        if let Some(lexeme) = self.peek() {
            match &lexeme.data {
                LexemeData::Integer(n) => panic!("TODO: create expression"),
                _ => panic!("TODO: error"),
            }
        } else {
            panic!("TODO: error")
        }
    }
}
