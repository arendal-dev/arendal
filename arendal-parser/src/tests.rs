use ast::{
    Binary, Expr, Expression, Q, Statement, TypeExpr, common::BinaryOp, position::EqNoPosition,
    symbol::TSymbol,
};

fn check_statements(input: &str, expected: Vec<Statement>) {
    let warnings = super::parse_statements(input).unwrap();
    warnings.value.assert_eq_nopos(&expected);
}

fn check_statement(input: &str, expected: Statement) {
    check_statements(input, vec![expected]);
}

fn check_expression(input: &str, expected: Expression) {
    check_statement(input, Statement::Expression(expected));
}

fn e(expr: Expr) -> Expression {
    expr.to_expression(ast::position::Position::NoPosition, None)
}

fn e_i64(value: i64) -> Expression {
    e(Expr::LitInteger(value.into()))
}

fn e_binary(expr1: Expression, op: BinaryOp, expr2: Expression) -> Expression {
    e(Expr::Binary(Binary {
        op,
        expr1: expr1.into(),
        expr2: expr2.into(),
    }))
}

fn e_add(expr1: Expression, expr2: Expression) -> Expression {
    e_binary(expr1, BinaryOp::Add, expr2)
}

fn e_add_i64(v1: i64, v2: i64) -> Expression {
    e_add(e_i64(v1), e_i64(v2))
}

fn tai(mut e: Expression) -> Expression {
    e.type_expr = Some(TypeExpr::Type(Q::of(TSymbol::new("Integer").unwrap())));
    e
}

#[test]
fn int_literal_expr() {
    check_expression("1234", e_i64(1234));
    check_expression("  1234 ", e_i64(1234));
    check_expression("\t \n 1234  \n\t", e_i64(1234));
}

#[test]
fn ann_int_literal_expr() {
    check_expression("1234 : Integer", tai(e_i64(1234)));
    check_expression("1234 :Integer", tai(e_i64(1234)));
    check_expression("1234:Integer", tai(e_i64(1234)));
}

#[test]
fn int_add() {
    check_expression("1 +2 ", e_add_i64(1, 2));
}
