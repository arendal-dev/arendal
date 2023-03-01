use super::{Error, Loc, Result};
use crate::{Enclosure, Errors, Expression, Lexeme, LexemeKind, Lexemes};
use ast::BinaryOp;

pub(crate) struct Parser {
    input: Lexemes,
    index: usize,
    errors: Errors,
}

impl Parser {
    pub(crate) fn new(input: Lexemes, index: usize) -> Parser {
        Parser {
            input,
            index,
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
    fn peek(&self) -> Option<Lexeme> {
        self.input.get(self.index)
    }

    // Consumes one lexer a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Lexeme> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Lexeme> {
        self.input.get(self.index + n)
    }

    // If the next lexeme maches the provided one, advances it and returns true
    fn match1(&mut self, kind: LexemeKind) -> bool {
        if let Some(lexeme) = self.peek() {
            if kind == *lexeme.kind() {
                self.consume();
                return true;
            }
        }
        false
    }

    // Parses a single expression, if any, consuming as many lexemes as needed.
    pub(crate) fn parse(mut self) -> (Result<Expression>, usize) {
        let result = match self.rule_expression() {
            Some(expr) => self.errors.to_result(expr),
            None => {
                self.errors
                    .add(Loc::none(), super::ExpressionExpectedError {});
                Err(self.errors)
            }
        };
        (result, self.index)
    }

    fn rule_expression(&mut self) -> Option<Expression> {
        self.rule_term()
    }

    fn rule_term(&mut self) -> Option<Expression> {
        let mut left = self.rule_factor()?;
        while let Some(lexeme) = self.peek() {
            let maybe = match lexeme.kind() {
                LexemeKind::Plus => Some(BinaryOp::Add),
                LexemeKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            if let Some(op) = maybe {
                self.consume();
                let right = self.rule_factor()?;
                left = Expression::binary(lexeme.loc(), op, left, right);
            } else {
                break;
            }
        }
        Some(left)
    }

    fn rule_factor(&mut self) -> Option<Expression> {
        let mut left = self.rule_primary()?;
        while let Some(lexeme) = self.peek() {
            let maybe = match lexeme.kind() {
                LexemeKind::Star => Some(BinaryOp::Mul),
                LexemeKind::Slash => Some(BinaryOp::Div),
                _ => None,
            };
            if let Some(op) = maybe {
                self.consume();
                let right = self.rule_primary()?;
                left = Expression::binary(lexeme.loc(), op, left, right);
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
                    Some(Expression::lit_integer(lexeme.loc(), n.clone()))
                }
                LexemeKind::Open(Enclosure::Parens) => {
                    self.consume();
                    let result = self.rule_expression();
                    if !self.match1(LexemeKind::Close(Enclosure::Parens)) {
                        self.add_error(&lexeme, Error::ParsingError);
                    }
                    result
                }
                _ => self.add_error(&lexeme, Error::ParsingError),
            }
        } else {
            None
        }
    }

    fn add_error(&mut self, lexeme: &Lexeme, error: Error) -> Option<Expression> {
        self.errors.add(lexeme.loc(), error);
        None
    }
}
