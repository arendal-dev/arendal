use core::ast::{BinaryOp, ExprBuilder, Expression};
use core::error::{Errors, Result};
use core::keyword::Keyword;
use core::symbol::Symbol;

use crate::lexer::{Lexeme, LexemeKind};
use crate::Enclosure;

use super::{Parser, ParserError};

fn builder(lexeme: &Lexeme) -> ExprBuilder {
    ExprBuilder::new(lexeme.loc())
}

// Parses a single expression, if any, consuming as many lexemes as needed.
pub(super) fn parse(parser: &mut Parser) -> Result<Expression> {
    rule_toplevelexpr(parser)
}

fn rule_toplevelexpr(parser: &mut Parser) -> Result<Expression> {
    if parser.match1(LexemeKind::Keyword(Keyword::Val)) {
        rule_assignment(parser)
    } else {
        rule_expression(parser)
    }
}

fn rule_assignment(parser: &mut Parser) -> Result<Expression> {
    let lvalue = get_lvalue(parser)?;
    if let Some(lexeme) = parser.consume_if_match(LexemeKind::Assignment) {
        let expr = rule_expression(parser)?;
        Ok(builder(&lexeme).assignment(lvalue, expr))
    } else {
        parser.err(ParserError::AssignmentExpended)
    }
}

fn get_lvalue(parser: &mut Parser) -> Result<Symbol> {
    if let Some(lexeme) = parser.peek() {
        if let LexemeKind::Id(id) = lexeme.kind() {
            return parser.ok(id.clone());
        }
    }
    parser.err(ParserError::LValueExpectedError)
}

fn rule_expression(parser: &mut Parser) -> Result<Expression> {
    rule_term(parser)
}

fn rule_term(parser: &mut Parser) -> Result<Expression> {
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
            left = builder(&lexeme).binary(op, left, right);
        } else {
            break;
        }
    }
    Ok(left)
}

fn rule_factor(parser: &mut Parser) -> Result<Expression> {
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
            left = builder(&lexeme).binary(op, left, right);
        } else {
            break;
        }
    }
    Ok(left)
}

fn rule_primary(parser: &mut Parser) -> Result<Expression> {
    if let Some(lexeme) = parser.peek() {
        match &lexeme.kind() {
            LexemeKind::Integer(n) => parser.ok(builder(&lexeme).lit_integer(n.clone())),
            LexemeKind::TypeId(id) => parser.ok(builder(&lexeme).tsymbol(id.clone())),
            LexemeKind::Id(id) => parser.ok(builder(&lexeme).symbol(id.clone())),
            LexemeKind::Open(Enclosure::Parens) => {
                parser.consume();
                let mut result = rule_expression(parser);
                if !parser.match1(LexemeKind::Close(Enclosure::Parens)) {
                    result = Errors::add_to(result, lexeme.loc(), ParserError::ParsingError);
                }
                result
            }
            _ => parser.expression_expected(),
        }
    } else {
        parser.expression_expected()
    }
}
