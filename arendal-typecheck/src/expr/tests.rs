use super::check;
use ast::bare::Expression;
use ast::error::Result;
use ast::{Loc, Type, TypedExpression};

fn int(value: i64) -> Expression {
    ast::bare::lit_integer(value.into())
}

fn add_int(v1: i64, v2: i64) -> Expression {
    ast::bare::binary(ast::BinaryOp::Add, int(v1), int(v2))
}

fn ok_type<L: Loc>(typed: Result<TypedExpression<L>>, t: Type) {
    assert_eq!(typed.unwrap().payload.loc_type, t);
}

#[test]
fn integer() {
    ok_type(check(&int(1234)), Type::Integer);
}

#[test]
fn sum() {
    ok_type(check(&add_int(1, 2)), Type::Integer);
}
