use super::Type;
use crate::env::Env;
use crate::error::Result;
use crate::symbol::Pkg;
use crate::typed;

fn check_module(input: &str) -> Result<typed::Module> {
    let parsed = crate::parser::parse(input)?;
    let env = Env::default();
    let path = Pkg::Local.empty();
    super::check(&env, &path, &parsed)
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
