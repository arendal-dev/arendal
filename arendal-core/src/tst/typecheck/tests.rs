use super::{Package, Type};
use crate::env::Env;
use crate::error::{Error, Result};
use crate::symbol::Pkg;

fn check_module(input: &str) -> Result<Package> {
    let parsed = crate::ast::parser::parse(Pkg::Local, input)?;
    let env = Env::default();
    super::check(&env, &parsed)
}

fn ok(input: &str) {
    check_module(input).unwrap();
}

fn error(input: &str) {
    if let Ok(_) = check_module(input) {
        panic!("Expected Error")
    }
}

fn expect_error(input: &str, error: &Error) {
    match check_module(input) {
        Ok(_) => panic!("Expected Error: {:?}", error),
        Err(e) => assert!(
            e.contains(error),
            "Expected Error: {:?} - Actual Error: {:?}",
            error,
            e
        ),
    }
}

fn mismatch(input: &str, expected: Type, actual: Type) {
    expect_error(input, &Error::type_mismatch(expected, actual))
}

fn ok_expression(input: &str, t: Type) {
    assert_eq!(*check_module(input).unwrap().expr.unwrap().borrow_type(), t);
}

fn ok_int(input: &str) {
    ok_expression(input, Type::Integer);
}

#[test]
fn integer() {
    ok_int("1234");
}

#[test]
fn add1() {
    ok_int("1 + 2");
}

#[test]
fn add2() {
    ok_int("1 + 2 + 3");
}

#[test]
fn sub1() {
    ok_int("1- 2");
}

#[test]
fn std_singleton() {
    ok_expression("True", Type::True);
}

#[test]
fn mismatch1() {
    mismatch("1 + True", Type::Integer, Type::True)
}

#[test]
fn mismatch2() {
    mismatch("1 && True", Type::Boolean, Type::Integer)
}

#[test]
fn conditionals() {
    ok_int("if True then 1 else 2");
    mismatch("if 0 then 1 else 2", Type::Boolean, Type::Integer);
    mismatch("if True then 1 else False", Type::Integer, Type::False);
}

#[test]
fn assignments() {
    //ok("let x = 1");
    ok("let x = 1\nlet y = x + 1");
}

#[test]
fn seq() {
    ok_int("True then False then 3")
}

#[test]
fn blocks() {
    ok_int("{ let x = 1\n x+2 }")
}

#[test]
fn only_one_expr() {
    expect_error("1\n2", &Error::OnlyOneExpressionAllowed);
    expect_error("{ 1\n2 }", &Error::OnlyOneExpressionAllowed)
}
