use super::Package;
use crate::env::{Env, Type};
use crate::error::{Error, Result};

fn check_module(input: &str) -> Result<Package> {
    let parsed = crate::ast::parser::parse(input)?;
    let env = Env::default();
    super::check(&env, &parsed)
}

fn error(input: &str, error: &Error) {
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
    error(input, &Error::type_mismatch(expected, actual))
}

fn ok_expression(input: &str, t: Type) {
    assert_eq!(
        *check_module(input)
            .unwrap()
            .expressions
            .iter()
            .next()
            .unwrap()
            .borrow_type(),
        t
    );
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
fn blocks() {
    ok_int("{ let x = 1\n x+2 }")
}
