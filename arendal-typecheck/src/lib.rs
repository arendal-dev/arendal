use ast::error::{Error, Errors, Result};
use ast::{Expression, Loc, Type, Typed, TypedExpression};

// 'static here means that L is owned
pub fn expression<L: Loc + 'static>(input: &Expression<L>) -> Result<TypedExpression<L>> {
    ExprChecker::new(input).check()
}

struct ExprChecker<'a, L: Loc> {
    input: &'a Expression<L>,
    errors: Errors,
}

impl<'a, L: Loc + 'static> ExprChecker<'a, L> {
    fn new(input: &'a Expression<L>) -> Self {
        ExprChecker {
            input,
            errors: Default::default(),
        }
    }

    fn check(mut self) -> Result<TypedExpression<L>> {
        match self.check_expr(self.input) {
            Some(typed) => Ok(typed),
            None => Err(self.errors),
        }
    }

    // Adds an error and returns `None` so that it can be used as tail call
    fn add_error(
        &mut self,
        expr: &'a Expression<L>,
        kind: TypeErrorKind,
    ) -> Option<TypedExpression<L>> {
        self.errors.add(TypeError::new(expr, kind));
        None
    }

    fn check_expr(&mut self, expr: &'a Expression<L>) -> Option<TypedExpression<L>> {
        match &expr.expr {
            ast::Expr::LitInteger(value) => Some(Expression::lit_integer(
                Typed::new(expr, Type::Integer),
                value.clone(),
            )),
            _ => self.add_error(&expr, TypeErrorKind::InvalidType),
        }
    }
}

#[derive(Debug)]
struct TypeError<L: Loc> {
    loc: L,
    kind: TypeErrorKind,
}

impl<L: Loc> TypeError<L> {
    fn new(expr: &Expression<L>, kind: TypeErrorKind) -> Self {
        TypeError {
            loc: expr.payload.clone(),
            kind,
        }
    }
}

#[derive(Debug)]
enum TypeErrorKind {
    InvalidType, // placeholder, temporary error
}

impl<L: Loc> Error for TypeError<L> {}

#[cfg(test)]
mod tests;
