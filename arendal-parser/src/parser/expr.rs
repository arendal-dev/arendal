use core::ast::{BinaryOp, Expression};
use core::error::Loc;
use core::id::TypeId;

use crate::lexer::LexemeKind;
use crate::Enclosure;

use super::{Parser, ParserError};

// Parses a single expression, if any, consuming as many lexemes as needed.
pub(super) fn parse(parser: &mut Parser) -> Option<Expression> {
    rule_expression(parser)
}

fn rule_expression(parser: &mut Parser) -> Option<Expression> {
    rule_term(parser)
}

fn rule_term(parser: &mut Parser) -> Option<Expression> {
    let mut left = rule_factor(parser)?;
    while let Some(lexeme) = parser.peek() {
        let maybe = match lexeme.kind() {
            LexemeKind::Plus => Some(BinaryOp::Add),
            LexemeKind::Minus => Some(BinaryOp::Sub),
            _ => None,
        };
        if let Some(op) = maybe {
            parser.consume();
            let right = rule_factor(parser)?;
            left = Expression::binary(lexeme.loc(), op, left, right);
        } else {
            break;
        }
    }
    Some(left)
}

fn rule_factor(parser: &mut Parser) -> Option<Expression> {
    let mut left = rule_primary(parser)?;
    while let Some(lexeme) = parser.peek() {
        let maybe = match lexeme.kind() {
            LexemeKind::Star => Some(BinaryOp::Mul),
            LexemeKind::Slash => Some(BinaryOp::Div),
            _ => None,
        };
        if let Some(op) = maybe {
            parser.consume();
            let right = rule_primary(parser)?;
            left = Expression::binary(lexeme.loc(), op, left, right);
        } else {
            break;
        }
    }
    Some(left)
}

fn rule_primary(parser: &mut Parser) -> Option<Expression> {
    if let Some(lexeme) = parser.peek() {
        match &lexeme.kind() {
            LexemeKind::Integer(n) => {
                parser.consume();
                Some(Expression::lit_integer(lexeme.loc(), n.clone()))
            }
            LexemeKind::TypeId(id) => lit_type(parser, lexeme.loc(), id.clone()),
            LexemeKind::Open(Enclosure::Parens) => {
                parser.consume();
                let result = rule_expression(parser);
                if !parser.match1(LexemeKind::Close(Enclosure::Parens)) {
                    parser.add_error(&lexeme, ParserError::ParsingError);
                }
                result
            }
            _ => parser.add_error(&lexeme, ParserError::ParsingError),
        }
    } else {
        None
    }
}

fn lit_type(parser: &mut Parser, loc: Loc, id: TypeId) -> Option<Expression> {
    // Very simple for now
    parser.consume();
    Some(Expression::lit_type(loc, id))
}
