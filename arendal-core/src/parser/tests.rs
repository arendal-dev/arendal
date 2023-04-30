use crate::ast::{BinaryOp, Module, ModuleItem};
use crate::ast::{ExprBuilder, Expression};
use crate::error::Loc;
use crate::symbol::{Symbol, TSymbol};

use super::{parse, ParserError};

const B: ExprBuilder = ExprBuilder::none();

fn check_module(input: &str, expected: Module) {
    assert_eq!(
        parse(input).unwrap(),
        expected,
        "Left=Actual; Right=Expected"
    )
}

fn check_expression(input: &str, expected: Expression) {
    check_module(input, Module::new(vec![ModuleItem::Expression(expected)]))
}

fn check_expressions(input: &str, expected: Vec<Expression>) {
    let items = expected
        .into_iter()
        .map(|e| ModuleItem::Expression(e))
        .collect();
    check_module(input, Module::new(items))
}

fn expect_one_error(input: &str, expected: ParserError) {
    match parse(input) {
        Ok(_) => panic!("Parsed correctly but expected {:?}", expected),
        Err(e) => assert!(
            e.is(expected.clone()),
            "Expected {:?} but error was {:?}",
            expected,
            e
        ),
    }
}

fn sym(symbol: &str) -> Symbol {
    Symbol::new(Loc::none(), symbol.into()).unwrap()
}

fn tsym(symbol: &str) -> TSymbol {
    TSymbol::new(Loc::none(), symbol.into()).unwrap()
}

fn x() -> Symbol {
    sym("x")
}

fn y() -> Symbol {
    sym("y")
}

fn e_i64(value: i64) -> Expression {
    B.lit_integer(value.into())
}

fn e_x() -> Expression {
    B.symbol(x())
}

fn e_y() -> Expression {
    B.symbol(y())
}

fn e_true() -> Expression {
    B.tsymbol(TSymbol::True)
}

fn e_false() -> Expression {
    B.tsymbol(TSymbol::False)
}

fn add(expr1: Expression, expr2: Expression) -> Expression {
    B.binary(BinaryOp::Add, expr1, expr2)
}

fn add_i64(value1: i64, value2: i64) -> Expression {
    add(e_i64(value1), e_i64(value2))
}

fn sub(expr1: Expression, expr2: Expression) -> Expression {
    B.binary(BinaryOp::Sub, expr1, expr2)
}

fn sub_i64(value1: i64, value2: i64) -> Expression {
    sub(e_i64(value1), e_i64(value2))
}

fn and(expr1: Expression, expr2: Expression) -> Expression {
    B.binary(BinaryOp::And, expr1, expr2)
}

fn or(expr1: Expression, expr2: Expression) -> Expression {
    B.binary(BinaryOp::Or, expr1, expr2)
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
fn assignment1() {
    check_expression("val x = 1", B.assignment(x(), e_i64(1)));
}

#[test]
fn assignment2() {
    check_expression("val x = y + 2", B.assignment(x(), add(e_y(), e_i64(2))));
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
    check_expressions("1\n2", vec![e_i64(1), e_i64(2)]);
}

#[test]
fn multiple2() {
    expect_one_error("1 2", ParserError::EndOfItemExpected)
}

#[test]
fn logical_ops() {
    check_expression(
        "True || False && True",
        or(e_true(), and(e_false(), e_true())),
    );
}
