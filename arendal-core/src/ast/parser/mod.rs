mod lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}

use super::{Assignment, BStmt, BinaryOp, Builder, Expr, Module, Package, Segment, TypeDefinition};
use crate::error::{Error, Loc, Result, L};
use crate::keyword::Keyword;
use crate::symbol::{Path, Pkg, Symbol, TSymbol};
use crate::visibility::Visibility;
use std::rc::Rc;

use lexer::{lex, Lexeme, LexemeKind, Lexemes, Separator};

// Parses the input as a package
pub fn parse(input: &str) -> Result<Package> {
    let module = parse_module(input)?;
    let mut package = Package {
        pkg: Pkg::Local,
        modules: Default::default(),
    };
    package.modules.insert(Path::empty(), module);
    Ok(package)
}

fn parse_module(input: &str) -> Result<Module> {
    let lexemes = lex(input)?;
    Parser::new(lexemes).parse()
}

type PResult<T> = Result<(T, Parser)>;
type EResult = PResult<L<Expr>>;
type BResult = PResult<BStmt>;
type TResult = PResult<TypeDefinition>;

fn map<T, F, U>(result: PResult<T>, f: F) -> PResult<U>
where
    F: FnOnce(T) -> U,
{
    match result {
        Ok((t, p)) => Ok((f(t), p)),
        Err(e) => Err(e),
    }
}

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

    fn peek_kind(&self) -> Option<&LexemeKind> {
        self.peek().map(|l| &l.kind)
    }

    // Returns the lexeme at the current index, if its kind equals the provided one.
    fn peek_if_kind(&self, kind: LexemeKind) -> Option<&Lexeme> {
        self.peek().filter(|l| kind == l.kind)
    }

    fn kind_equals(&self, kind: LexemeKind) -> bool {
        self.peek_if_kind(kind).is_some()
    }

    fn is_keyword(&self, keyword: Keyword) -> bool {
        self.kind_equals(LexemeKind::Keyword(keyword))
    }

    // Returns whether the current lexeme is EOS (end of statement)
    // I.e., either end of the input or a newline separator
    fn is_eos(&self) -> bool {
        match self.peek() {
            Some(lexeme) => lexeme.separator == Separator::NewLine,
            None => true,
        }
    }

    fn parse(self) -> Result<Module> {
        let mut module = Module::default();
        let mut parser = self;
        while !parser.is_done() {
            (_, parser) = parser.rule_statement(&mut module)?;
        }
        Ok(module)
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

    fn err<T>(&self, error: Error) -> Result<T> {
        self.loc().err(error)
    }

    fn expression_expected<T>(&self) -> Result<T> {
        self.err(Error::ExpressionExpected)
    }

    fn keyword_expected(&self, keyword: Keyword) -> Result<Parser> {
        if self.is_keyword(keyword) {
            Ok(self.advance())
        } else {
            self.err(Error::KeywordExpected(keyword))
        }
    }

    fn expect_eos<O>(&self, o: O) -> PResult<()>
    where
        O: FnOnce() -> (),
    {
        if self.is_eos() {
            self.ok(o())
        } else {
            self.err(Error::EndOfItemExpected)
        }
    }

    fn builder(&self) -> Builder {
        Builder::new(self.loc())
    }

    fn rule_statement(&self, module: &mut Module) -> PResult<()> {
        let visibility = if self.is_keyword(Keyword::Pub) {
            Visibility::Exported
        } else if self.is_keyword(Keyword::Pkg) {
            Visibility::Package
        } else {
            Visibility::Module
        };
        let next = self.advance();
        let parser = if visibility == Visibility::Module {
            self
        } else {
            &next
        };
        if parser.is_keyword(Keyword::Type) {
            let (dfn, parser) = self.advance().rule_typedef()?;
            parser.expect_eos(|| module.types.push(dfn))
        } else if parser.is_keyword(Keyword::Let) {
            let (a, parser) = self.advance().rule_assignment()?;
            parser.expect_eos(|| module.assignments.push(a.to_lv(visibility)))
        } else {
            if visibility == Visibility::Module {
                let (expr, parser) = self.rule_expression()?;
                parser.expect_eos(|| module.exprs.push(expr))
            } else {
                parser.err(Error::ExpressionNotExpected)
            }
        }
    }

    fn rule_typedef(&self) -> TResult {
        let symbol = match self.peek_kind() {
            Some(LexemeKind::TSymbol(symbol)) => Ok(symbol.clone()),
            _ => self.err(Error::TSymbolAfterTypeExpected),
        }?;
        self.advance().ok(self.builder().singleton(symbol))
    }

    fn rule_bstatement(&self) -> BResult {
        if self.is_keyword(Keyword::Let) {
            map(self.advance().rule_assignment(), |a| a.to_bstmt())
        } else {
            self.rule_expression().map(|(e, p)| (e.to_bstmt(), p))
        }
    }

    fn rule_assignment(&self) -> PResult<L<Assignment>> {
        let (lvalue, parser) = self.get_lvalue()?;
        if parser.kind_equals(LexemeKind::Assignment) {
            let (expr, next) = parser.advance().rule_expression()?;
            next.ok(parser.builder().assignment(lvalue, expr))
        } else {
            parser.err(Error::AssignmentExpected)
        }
    }

    fn get_lvalue(&self) -> PResult<Symbol> {
        if let Some(lexeme) = self.peek() {
            if let LexemeKind::Symbol(id) = &lexeme.kind {
                return self.advance().ok(id.clone());
            }
        }
        self.err(Error::LValueExpected)
    }

    fn rule_expression(&self) -> EResult {
        let (mut left, mut parser) = self.rule_expr()?;
        while parser.is_keyword(Keyword::Then) {
            let (right, p2) = parser.advance().rule_expr()?;
            (left, parser) = p2.ok(parser.builder().seq(left, right))?;
        }
        parser.ok(left)
    }

    fn rule_expr(&self) -> EResult {
        if self.is_keyword(Keyword::If) {
            self.rule_conditional()
        } else {
            self.rule_subexpr()
        }
    }

    fn rule_conditional(&self) -> EResult {
        let (expr, parser2) = self.advance().rule_expr()?;
        let (then, parser3) = parser2.keyword_expected(Keyword::Then)?.rule_expr()?;
        let (otherwise, parser4) = parser3.keyword_expected(Keyword::Else)?.rule_expr()?;
        parser4.ok(self.builder().conditional(expr, then, otherwise))
    }

    fn rule_subexpr(&self) -> EResult {
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
                LexemeKind::TSymbol(symbol) => self.rule_q(Segment::Type(symbol.clone())),
                LexemeKind::Symbol(symbol) => self.rule_q(Segment::Symbol(symbol.clone())),
                LexemeKind::Open(Enclosure::Parens) => {
                    let (expr, next) = self.advance().rule_expression()?;
                    if !next.kind_equals(LexemeKind::Close(Enclosure::Parens)) {
                        next.err(Error::CloseExpected(Enclosure::Parens))
                    } else {
                        next.advance().ok(expr)
                    }
                }
                LexemeKind::Open(Enclosure::Curly) => {
                    let mut parser = self.advance();
                    if parser.kind_equals(LexemeKind::Close(Enclosure::Curly)) {
                        parser
                            .advance()
                            .ok(self.builder().tsymbol(Vec::default(), TSymbol::None))
                    } else {
                        let mut stmt;
                        let mut stmts: Vec<BStmt> = Vec::default();
                        loop {
                            if parser.is_done() {
                                return parser.err(Error::CloseExpected(Enclosure::Curly));
                            }
                            (stmt, parser) = parser.rule_bstatement()?;
                            stmts.push(stmt);
                            if parser.kind_equals(LexemeKind::Close(Enclosure::Curly)) {
                                parser = parser.advance();
                                break;
                            } else if !parser.is_eos() {
                                return parser.err(Error::EndOfItemExpected);
                            }
                        }
                        parser.ok(self.builder().block(stmts))
                    }
                }
                _ => self.expression_expected(),
            }
        } else {
            self.expression_expected()
        }
    }

    fn rule_q(&self, segment: Segment) -> EResult {
        let segments = Vec::default();
        self.rule_q_add(self, segments, segment)
    }

    fn rule_q_peek(&self, initial: &Parser, segments: Vec<Segment>) -> EResult {
        if let Some(lex) = self.peek() {
            if lex.kind == LexemeKind::PathSeparator {
                if lex.separator == Separator::Nothing {
                    let parser = self.advance();
                    match parser.peek_kind() {
                        Some(LexemeKind::Symbol(s)) => {
                            parser.rule_q_add(initial, segments, Segment::Symbol(s.clone()))
                        }
                        Some(LexemeKind::TSymbol(s)) => {
                            parser.rule_q_add(initial, segments, Segment::Type(s.clone()))
                        }
                        _ => parser.err(Error::ParsingError),
                    }
                } else {
                    self.err(Error::ParsingError)
                }
            } else {
                self.rule_q_close(initial, segments)
            }
        } else {
            self.rule_q_close(initial, segments)
        }
    }

    fn rule_q_add(
        &self,
        initial: &Parser,
        mut segments: Vec<Segment>,
        segment: Segment,
    ) -> EResult {
        segments.push(segment);
        self.advance().rule_q_peek(initial, segments)
    }

    fn rule_q_close(&self, initial: &Parser, mut segments: Vec<Segment>) -> EResult {
        match segments.pop().unwrap() {
            Segment::Symbol(symbol) => self.ok(initial.builder().symbol(segments, symbol)),
            Segment::Type(symbol) => self.ok(initial.builder().tsymbol(segments, symbol)),
        }
    }
}

#[cfg(test)]
mod tests;
