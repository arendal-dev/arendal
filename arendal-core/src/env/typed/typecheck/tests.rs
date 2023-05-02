use super::{Module, Type};
use crate::env::Env;
use crate::error::{Error, Result};
use crate::symbol::Pkg;

fn check_module(input: &str) -> Result<Module> {
    let parsed = crate::parser::parse(input)?;
    let env = Env::default();
    let path = Pkg::Local.empty();
    super::check(&env, &path, &parsed)
}

fn error(input: &str, error: &Error) {
    match check_module(input) {
        Ok(_) => panic!("Expected error: {:?}", error),
        Err(e) => assert!(
            e.contains(error),
            "Expected error: {:?}\nActual: {:?}",
            error,
            e
        ),
    }
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
    error("1 + True", &Error::type_mismatch(Type::Integer, Type::True))
}

#[test]
fn mismatch2() {
    error(
        "1 && True",
        &Error::type_mismatch(Type::Boolean, Type::Integer),
    )
}
