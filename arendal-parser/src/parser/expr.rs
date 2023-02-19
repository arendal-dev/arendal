use super::{Error, ErrorKind};
use crate::{Errors, Expression, Indentation, LexemeKind, LexemeRef, Lexemes, Result};
use ast::BinaryOp;
use std::rc::Rc;

pub(crate) struct Parser {
    input: Rc<Lexemes>,
    start: usize, // Start index for the expression
    index: usize,
    errors: Errors,
}

impl Parser {
    pub(crate) fn new(input: Rc<Lexemes>, start: usize) -> Parser {
        Parser {
            input,
            start,
            index: start,
            errors: Default::default(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        !self.input.contains(self.index)
    }

    // Consumes one lexer, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the lexer at the current index, if any
    fn peek(&self) -> Option<LexemeRef> {
        self.input.get(self.index)
    }

    // Consumes one lexer a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<LexemeRef> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<LexemeRef> {
        self.input.get(self.index + n)
    }

    // Parses a single expression, if any, consuming as many tokens as needed.
    pub(crate) fn parse(mut self) -> Result<Option<Expression>> {
        let maybe = self.rule_expression();
        self.errors.to_result(maybe)
    }

    fn rule_expression(&mut self) -> Option<Expression> {
        self.rule_term()
    }

    fn rule_term(&mut self) -> Option<Expression> {
        let mut left = self.rule_primary()?;
        while let Some(lexeme) = self.peek() {
            let maybe = match lexeme.kind() {
                LexemeKind::Plus => Some(BinaryOp::Add),
                LexemeKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            if let Some(op) = maybe {
                self.consume();
                let right = self.rule_primary()?;
                left = Expression::binary(lexeme.pos(), op, left, right);
            } else {
                break;
            }
        }
        Some(left)
    }

    fn rule_primary(&mut self) -> Option<Expression> {
        if let Some(lexeme) = self.peek() {
            match &lexeme.kind() {
                LexemeKind::Integer(n) => {
                    self.consume();
                    Some(Expression::lit_integer(lexeme.pos(), n.clone()))
                }
                _ => self.add_error(&lexeme, ErrorKind::ParsingError),
            }
        } else {
            None
        }
    }

    fn add_error(&mut self, lexeme: &LexemeRef, kind: ErrorKind) -> Option<Expression> {
        self.errors.add(Error::new(lexeme, kind));
        None
    }
}
