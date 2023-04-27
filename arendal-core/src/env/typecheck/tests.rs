use super::Type;
use crate::ast;
use crate::env::Env;
use crate::symbol::Pkg;

const B: ast::ExprBuilder = ast::ExprBuilder::none();

fn ok_type(expr: ast::Expression, t: Type) {
    let env = Env::default();
    let path = Pkg::Local.empty();
    let mut module = ast::Module::default();
    module.push(ast::ModuleItem::Expression(expr));
    assert_eq!(
        *super::check(&env, &path, &module)
            .unwrap()
            .expressions
            .iter()
            .next()
            .unwrap()
            .borrow_type(),
        t
    );
}

fn ok_int(expr: ast::Expression) {
    ok_type(expr, Type::Integer);
}

#[test]
fn integer() {
    ok_int(B.lit_i64(1234));
}

#[test]
fn add1() {
    ok_int(B.add_i64(1, 2));
}

#[test]
fn add2() {
    ok_int(B.add(B.add_i64(1, 2), B.lit_i64(3)));
}

#[test]
fn sub1() {
    ok_int(B.sub_i64(1, 2));
}
