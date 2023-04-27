use core::ast::{BinaryOp, ExprBuilder, Expression, Module, ModuleItem};
use core::error::{Error, Errors, Loc, Result};
use core::keyword::Keyword;
use core::symbol::Symbol;
use std::rc::Rc;

use crate::lexer::{lex, Lexeme, LexemeKind, Lexemes};
use crate::Enclosure;

// Parses the input as a module
pub fn parse(input: &str) -> Result<Module> {
    let lexemes = lex(input)?;
    Parser::new(lexemes).parse()
}

type PResult<T> = Result<(T, Parser)>;
type EResult = PResult<Expression>;

#[derive(Clone)]
struct Parser {
    input: Rc<Lexemes>,
    index: usize,
}

impl Parser {
    fn new(input: Lexemes) -> Self {
        Self {
            input: Rc::new(input),
            index: 0,
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.index >= self.input.len()
    }

    // Returns a new parser that has advanced one lexeme.
    fn advance(&self) -> Self {
        assert!(!self.is_done());
        Self {
            input: self.input.clone(),
            index: self.index + 1,
        }
    }

    // Returns the lexeme at the current index, if any
    fn peek(&self) -> Option<&Lexeme> {
        self.input.get(self.index)
    }

    // Returns the lexeme at the current index, if its kind equals the provided one.
    fn peek_if_kind(&self, kind: LexemeKind) -> Option<&Lexeme> {
        self.peek().filter(|l| kind == *l.kind())
    }

    fn kind_equals(&self, kind: LexemeKind) -> bool {
        self.peek_if_kind(kind).is_some()
    }

    fn parse(self) -> Result<Module> {
        let mut module = Module::default();
        let mut parser = self;
        while !parser.is_done() {
            let expr: Expression;
            (expr, parser) = parser.rule_toplevelexpr()?;
            module.push(ModuleItem::Expression(expr))
        }
        Ok(module)
    }

    fn ok<T>(&self, value: T) -> PResult<T> {
        Ok((value, self.clone()))
    }

    fn loc(&self) -> Loc {
        let len = self.input.len(); // we now the vector is not empty
        self.input[if self.index < len { self.index } else { len }].loc()
    }

    fn err<T>(&self, error: ParserError) -> Result<T> {
        Err(Errors::new(self.loc(), error))
    }

    fn expression_expected<T>(&self) -> Result<T> {
        self.err(ParserError::ExpressionExpectedError)
    }

    fn builder(&self) -> ExprBuilder {
        ExprBuilder::new(self.loc())
    }

    fn rule_toplevelexpr(&self) -> EResult {
        if self.kind_equals(LexemeKind::Keyword(Keyword::Val)) {
            self.advance().rule_assignment()
        } else {
            self.rule_expression()
        }
    }

    fn rule_assignment(&self) -> EResult {
        let (lvalue, parser) = self.get_lvalue()?;
        if parser.kind_equals(LexemeKind::Assignment) {
            let (expr, next) = parser.advance().rule_expression()?;
            next.ok(parser.builder().assignment(lvalue, expr))
        } else {
            parser.err(ParserError::AssignmentExpended)
        }
    }

    fn get_lvalue(&self) -> PResult<Symbol> {
        if let Some(lexeme) = self.peek() {
            if let LexemeKind::Id(id) = lexeme.kind() {
                return self.advance().ok(id.clone());
            }
        }
        self.err(ParserError::LValueExpectedError)
    }

    fn rule_expression(&self) -> EResult {
        self.rule_term()
    }

    fn rule_term(&self) -> EResult {
        let (mut left, mut parser) = self.rule_factor()?;
        while !parser.is_done() {
            let lexeme = parser.peek().unwrap();
            match lexeme.kind() {
                LexemeKind::Plus => (left, parser) = parser.advance_term(left, BinaryOp::Add)?,
                LexemeKind::Minus => (left, parser) = parser.advance_term(left, BinaryOp::Sub)?,
                _ => break,
            }
        }
        parser.ok(left)
    }

    fn advance_term(&self, left: Expression, op: BinaryOp) -> EResult {
        let (right, parser) = self.advance().rule_factor()?;
        parser.ok(self.builder().binary(op, left, right))
    }

    fn rule_factor(&self) -> EResult {
        let (mut left, mut parser) = self.rule_primary()?;
        while !parser.is_done() {
            let lexeme = parser.peek().unwrap();
            match lexeme.kind() {
                LexemeKind::Star => (left, parser) = parser.advance_factor(left, BinaryOp::Mul)?,
                LexemeKind::Slash => (left, parser) = parser.advance_factor(left, BinaryOp::Div)?,
                _ => break,
            }
        }
        parser.ok(left)
    }

    fn advance_factor(&self, left: Expression, op: BinaryOp) -> EResult {
        let (right, parser) = self.advance().rule_primary()?;
        parser.ok(self.builder().binary(op, left, right))
    }

    fn rule_primary(&self) -> EResult {
        if let Some(lexeme) = self.peek() {
            match &lexeme.kind() {
                LexemeKind::Integer(n) => self.advance().ok(self.builder().lit_integer(n.clone())),
                LexemeKind::TypeId(id) => self.advance().ok(self.builder().tsymbol(id.clone())),
                LexemeKind::Id(id) => self.advance().ok(self.builder().symbol(id.clone())),
                LexemeKind::Open(Enclosure::Parens) => {
                    let (expr, next) = self.advance().rule_expression()?;
                    if !next.kind_equals(LexemeKind::Close(Enclosure::Parens)) {
                        next.err(ParserError::ParsingError)
                    } else {
                        next.advance().ok(expr)
                    }
                }
                _ => self.expression_expected(),
            }
        } else {
            self.expression_expected()
        }
    }
}

#[derive(Debug)]
enum ParserError {
    ExpressionExpectedError,
    LValueExpectedError,
    AssignmentExpended,
    ParsingError, // placeholder, temporary error
}

impl Error for ParserError {}

#[cfg(test)]
mod tests;
