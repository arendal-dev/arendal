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

fn str_symbol(symbol: &str) -> Symbol {
    Symbol::new(Loc::none(), symbol.into()).unwrap()
}

fn str_tsymbol(symbol: &str) -> TSymbol {
    TSymbol::new(Loc::none(), symbol.into()).unwrap()
}

fn x() -> Symbol {
    str_symbol("x")
}

fn y() -> Symbol {
    str_symbol("y")
}

fn x_expr() -> Expression {
    B.symbol(x())
}

fn y_expr() -> Expression {
    B.symbol(y())
}

fn true_expr() -> Expression {
    B.lit_true()
}

fn false_expr() -> Expression {
    B.lit_false()
}

#[test]
fn int_literal_expr() {
    check_expression("1234", B.lit_i64(1234));
}

#[test]
fn add1() {
    check_expression("1+2", B.add_i64(1, 2));
}

#[test]
fn add2() {
    check_expression("1 + 2", B.add_i64(1, 2));
}

#[test]
fn add3() {
    check_expression("\t1 + 2", B.add_i64(1, 2));
}

#[test]
fn add4() {
    check_expression("1 + 2 + 3", B.add(B.add_i64(1, 2), B.lit_i64(3)));
}

#[test]
fn add5() {
    check_expression(
        "1 +\t2 + 3\n+ 4",
        B.add(B.add(B.add_i64(1, 2), B.lit_i64(3)), B.lit_i64(4)),
    );
}

#[test]
fn sub1() {
    check_expression("1 - 2 + 1", B.add(B.sub_i64(1, 2), B.lit_i64(1)));
}

#[test]
fn lit_type() {
    check_expression("  True ", B.tsymbol(str_tsymbol("True")));
}

#[test]
fn add_id() {
    check_expression("1 +x", B.add(B.lit_i64(1), x_expr()));
}

#[test]
fn assignment1() {
    check_expression("val x = 1", B.assignment(str_symbol("x"), B.lit_i64(1)));
}

#[test]
fn assignment2() {
    check_expression(
        "val x = y + 2",
        B.assignment(str_symbol("x"), B.add(y_expr(), B.lit_i64(2))),
    );
}

#[test]
fn parens1() {
    check_expression(
        "(1 + 2) * 2",
        B.binary(BinaryOp::Mul, B.add_i64(1, 2), B.lit_i64(2)),
    );
}

#[test]
fn multiple1() {
    check_expressions("1\n2", vec![B.lit_i64(1), B.lit_i64(2)]);
}

#[test]
fn multiple2() {
    expect_one_error("1 2", ParserError::EndOfItemExpected)
}

#[test]
fn logical_ops() {
    check_expression(
        "True || False && True",
        B.or(true_expr(), B.and(false_expr(), true_expr())),
    );
}
