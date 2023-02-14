use super::check;
use ast::bare::*;
use ast::error::Result;
use ast::{Loc, Type, TypedExpression};

fn ok_type<L: Loc>(typed: Result<TypedExpression<L>>, t: Type) {
    assert_eq!(typed.unwrap().payload.loc_type, t);
}

#[test]
fn integer() {
    ok_type(check(&lit_i64(1234)), Type::Integer);
}

#[test]
fn sum() {
    ok_type(check(&add_i64(1, 2)), Type::Integer);
}
