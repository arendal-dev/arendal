use crate::ast::{Assignment, BinaryOp, Expr, Module, TypeDefinition};
use crate::ast::{Builder, Segment};
use crate::error::{Loc, L};
use crate::symbol::{self, Symbol, TSymbol};
use crate::visibility::Visibility;

use super::{parse, Enclosure, Error};

const B: Builder = Builder::none();

struct Test {
    expected: Module,
}

impl Test {
    fn new() -> Self {
        Self {
            expected: Default::default(),
        }
    }

    fn check(self, input: &str) {
        let package = parse(input).unwrap();
        let module = package.modules.iter().next().unwrap().1;
        assert_eq!(module, &self.expected, "Left=Actual; Right=Expected")
    }

    fn expr(mut self, expr: L<Expr>) -> Self {
        self.expected.exprs.push(expr);
        self
    }

    fn v_let(mut self, visibility: Visibility, symbol: Symbol, expr: L<Expr>) -> Self {
        self.expected
            .assignments
            .push(B.assignment(symbol, expr).to_lv(visibility));
        self
    }

    fn m_let(self, symbol: Symbol, expr: L<Expr>) -> Self {
        self.v_let(Visibility::Module, symbol, expr)
    }

    fn type_dfn(mut self, dfn: TypeDefinition) -> Self {
        self.expected.types.push(dfn);
        self
    }
}

fn check_expression(input: &str, expected: L<Expr>) {
    Test::new().expr(expected).check(input)
}

fn check_qsymbol(input: &str, segments: Vec<Segment>, symbol: Symbol) {
    check_expression(input, B.symbol(segments, symbol));
}

fn check_qtype(input: &str, segments: Vec<Segment>, symbol: TSymbol) {
    check_expression(input, B.tsymbol(segments, symbol));
}

fn expect_error(input: &str, expected: &Error) {
    match parse(input) {
        Ok(_) => panic!("Parsed correctly but expected {:?}", expected),
        Err(e) => assert!(
            e.contains(&expected),
            "Expected {:?} but error was {:?}",
            expected,
            e
        ),
    }
}

fn sym(symbol: &str) -> Symbol {
    Symbol::new(&Loc::None, symbol.into()).unwrap()
}

fn ssym(symbol: &str) -> Segment {
    Segment::Symbol(sym(symbol))
}

fn tsym(symbol: &str) -> TSymbol {
    TSymbol::new(&Loc::None, symbol.into()).unwrap()
}

fn stsym(symbol: &str) -> Segment {
    Segment::Type(tsym(symbol))
}

fn x() -> Symbol {
    sym("x")
}

fn y() -> Symbol {
    sym("y")
}

fn e_i64(value: i64) -> L<Expr> {
    B.lit_integer(value.into())
}

fn e_x() -> L<Expr> {
    B.symbol(Vec::default(), x())
}

fn e_y() -> L<Expr> {
    B.symbol(Vec::default(), y())
}

fn e_none() -> L<Expr> {
    B.tsymbol(Vec::default(), symbol::NONE.clone())
}

fn e_true() -> L<Expr> {
    B.tsymbol(Vec::default(), symbol::TRUE.clone())
}

fn e_false() -> L<Expr> {
    B.tsymbol(Vec::default(), symbol::FALSE.clone())
}

fn add(expr1: L<Expr>, expr2: L<Expr>) -> L<Expr> {
    B.binary(BinaryOp::Add, expr1, expr2)
}

fn add_i64(value1: i64, value2: i64) -> L<Expr> {
    add(e_i64(value1), e_i64(value2))
}

fn sub(expr1: L<Expr>, expr2: L<Expr>) -> L<Expr> {
    B.binary(BinaryOp::Sub, expr1, expr2)
}

fn sub_i64(value1: i64, value2: i64) -> L<Expr> {
    sub(e_i64(value1), e_i64(value2))
}

fn and(expr1: L<Expr>, expr2: L<Expr>) -> L<Expr> {
    B.binary(BinaryOp::And, expr1, expr2)
}

fn or(expr1: L<Expr>, expr2: L<Expr>) -> L<Expr> {
    B.binary(BinaryOp::Or, expr1, expr2)
}

fn seq_i64(value1: i64, value2: i64) -> L<Expr> {
    B.seq(e_i64(value1), e_i64(value2))
}

pub fn b_let(symbol: Symbol, expr: L<Expr>) -> L<Assignment> {
    B.assignment(symbol, expr)
}

fn check_type(input: &str, expected: TypeDefinition) {
    Test::new().type_dfn(expected).check(input)
}

fn singleton(symbol: &str) -> TypeDefinition {
    B.singleton(tsym(symbol))
}

#[test]
fn int_literal_expr() {
    check_expression("1234", e_i64(1234));
}

#[test]
fn add1() {
    check_expression("1+2", add_i64(1, 2));
}

#[test]
fn add2() {
    check_expression("1 + 2", add_i64(1, 2));
}

#[test]
fn add3() {
    check_expression("\t1 + 2", add_i64(1, 2));
}

#[test]
fn add4() {
    check_expression("1 + 2 + 3", add(add_i64(1, 2), e_i64(3)));
}

#[test]
fn add5() {
    check_expression(
        "1 +\t2 + 3\n+ 4",
        add(add(add_i64(1, 2), e_i64(3)), e_i64(4)),
    );
}

#[test]
fn sub1() {
    check_expression("1 - 2 + 1", add(sub_i64(1, 2), e_i64(1)));
}

#[test]
fn lit_type() {
    check_expression("  True ", e_true());
}

#[test]
fn add_id() {
    check_expression("1 +x", add(e_i64(1), e_x()));
}

#[test]
fn assignment() {
    Test::new().m_let(x(), e_i64(1)).check("let x = 1");
    Test::new()
        .m_let(x(), add(e_y(), e_i64(2)))
        .check("let x = y + 2");
}

#[test]
fn parens1() {
    check_expression(
        "(1 + 2) * 2",
        B.binary(BinaryOp::Mul, add_i64(1, 2), e_i64(2)),
    );
}

#[test]
fn multiple1() {
    Test::new().expr(e_i64(1)).expr(e_i64(2)).check("1\n2")
}

#[test]
fn multiple2() {
    expect_error("1 2", &Error::EndOfItemExpected)
}

#[test]
fn logical_ops() {
    check_expression(
        "True || False && True",
        or(e_true(), and(e_false(), e_true())),
    );
}

#[test]
fn seq() {
    check_expression("1 then 2", seq_i64(1, 2));
    check_expression("1 then 2 then 3", B.seq(seq_i64(1, 2), e_i64(3)));
}

#[test]
fn blocks() {
    check_expression("{ }", e_none());
    check_expression("{ 1 }", e_i64(1));
    check_expression("{ 1\n2 }", B.block(vec![], vec![e_i64(1), e_i64(2)]));
    check_expression(
        "{ let x = 1\n x+2 }",
        B.block(vec![b_let(x(), e_i64(1))], vec![add(e_x(), e_i64(2))]),
    );
    expect_error("{ 1 2 }", &Error::EndOfItemExpected);
    expect_error("{ 1\n 2 ", &Error::CloseExpected(Enclosure::Curly))
}

#[test]
fn qsymbols() {
    check_qsymbol("x", Vec::default(), x());
    check_qsymbol("x::y", vec![ssym("x")], y());
    check_qsymbol("x::z::y", vec![ssym("x"), ssym("z")], y());
    check_qtype("x::y::A", vec![ssym("x"), ssym("y")], tsym("A"));
    check_qtype("x::Y::A", vec![ssym("x"), stsym("Y")], tsym("A"));
}

#[test]
fn typedef_singleton() {
    check_type("type Red", singleton("Red"));
}
