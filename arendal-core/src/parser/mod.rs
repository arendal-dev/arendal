mod lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}

use crate::ast::{BinaryOp, ExprBuilder, Expression, Module, ModuleItem};
use crate::error::{Error, Loc, Result};
use crate::keyword::Keyword;
use crate::symbol::Symbol;
use std::rc::Rc;

use lexer::{lex, Lexeme, LexemeKind, Lexemes, Separator};

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
        self.peek().filter(|l| kind == l.kind)
    }

    fn kind_equals(&self, kind: LexemeKind) -> bool {
        self.peek_if_kind(kind).is_some()
    }

    // Returns whether the current lexeme is EOI (end of item)
    // I.e., either end of the input or a newline separator
    fn is_eoi(&self) -> bool {
        match self.peek() {
            Some(lexeme) => lexeme.separator == Separator::NewLine,
            None => true,
        }
    }

    fn parse(self) -> Result<Module> {
        let mut items = Vec::default();
        let mut parser = self;
        while !parser.is_done() {
            let item: ModuleItem;
            (item, parser) = parser.rule_moduleitem()?;
            items.push(item)
        }
        Ok(Module::new(items))
    }

    fn ok<T>(&self, value: T) -> PResult<T> {
        Ok((value, self.clone()))
    }

    fn loc(&self) -> Loc {
        let len = self.input.len(); // we know the vector is not empty
        self.input[if self.index < len {
            self.index
        } else {
            len - 1
        }]
        .loc()
    }

    fn err<T>(&self, error: ParserError) -> Result<T> {
        Error::err(self.loc(), error)
    }

    fn expression_expected<T>(&self) -> Result<T> {
        self.err(ParserError::ExpressionExpected)
    }

    fn builder(&self) -> ExprBuilder {
        ExprBuilder::new(self.loc())
    }

    fn rule_moduleitem(&self) -> PResult<ModuleItem> {
        let (expr, parser) = self.rule_toplevelexpr()?;
        if self.is_eoi() {
            parser.ok(ModuleItem::Expression(expr))
        } else {
            parser.err(ParserError::EndOfItemExpected)
        }
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
            parser.err(ParserError::AssignmentExpected)
        }
    }

    fn get_lvalue(&self) -> PResult<Symbol> {
        if let Some(lexeme) = self.peek() {
            if let LexemeKind::Id(id) = &lexeme.kind {
                return self.advance().ok(id.clone());
            }
        }
        self.err(ParserError::LValueExpected)
    }

    fn rule_expression(&self) -> EResult {
        self.rule_logterm()
    }

    fn binary_rule<O, F>(&self, op: O, rule: F) -> EResult
    where
        O: Fn(&LexemeKind) -> Option<BinaryOp>,
        F: Fn(&Parser) -> EResult,
    {
        let (mut left, mut parser) = rule(self)?;
        while let Some(bop) = parser.peek().and_then(|l| op(&l.kind)) {
            let (right, p2) = rule(&parser.advance())?;
            (left, parser) = p2.ok(parser.builder().binary(bop, left, right))?;
        }
        parser.ok(left)
    }

    fn rule_logterm(&self) -> EResult {
        let op = |k: &LexemeKind| match k {
            LexemeKind::LogicalOr => Some(BinaryOp::Or),
            _ => None,
        };
        self.binary_rule(op, Self::rule_logfactor)
    }

    fn rule_logfactor(&self) -> EResult {
        let op = |k: &LexemeKind| match k {
            LexemeKind::LogicalAnd => Some(BinaryOp::And),
            _ => None,
        };
        self.binary_rule(op, Self::rule_equality)
    }

    fn rule_equality(&self) -> EResult {
        let op = |k: &LexemeKind| match k {
            LexemeKind::Equals => Some(BinaryOp::Eq),
            LexemeKind::NotEquals => Some(BinaryOp::NEq),
            _ => None,
        };
        self.binary_rule(op, Self::rule_comparison)
    }

    fn rule_comparison(&self) -> EResult {
        let op = |k: &LexemeKind| match k {
            LexemeKind::Greater => Some(BinaryOp::GT),
            LexemeKind::GreaterOrEq => Some(BinaryOp::GE),
            LexemeKind::Less => Some(BinaryOp::LT),
            LexemeKind::LessOrEq => Some(BinaryOp::LE),
            _ => None,
        };
        self.binary_rule(op, Self::rule_term)
    }

    fn rule_term(&self) -> EResult {
        let op = |k: &LexemeKind| match k {
            LexemeKind::Plus => Some(BinaryOp::Add),
            LexemeKind::Minus => Some(BinaryOp::Sub),
            _ => None,
        };
        self.binary_rule(op, Self::rule_factor)
    }

    fn rule_factor(&self) -> EResult {
        let op = |k: &LexemeKind| match k {
            LexemeKind::Star => Some(BinaryOp::Mul),
            LexemeKind::Slash => Some(BinaryOp::Div),
            _ => None,
        };
        self.binary_rule(op, Self::rule_primary)
    }

    fn rule_primary(&self) -> EResult {
        if let Some(lexeme) = self.peek() {
            match &lexeme.kind {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    // Tokenizer
    UnexpectedChar(char),
    // Lexer
    InvalidClose(Enclosure),
    UnexpectedToken,
    // Parser
    ExpressionExpected,
    LValueExpected,
    AssignmentExpected,
    EndOfItemExpected,
    ParsingError, // placeholder, temporary error
}

#[cfg(test)]
mod tests;
