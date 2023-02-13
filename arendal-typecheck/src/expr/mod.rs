use super::{TypeError, TypeErrorKind};
use ast::error::{Errors, Result};
use ast::{Expression, Loc, Type, TypedExpression, TypedLoc};

// 'static here means that L is owned
pub(crate) fn check<L: Loc + 'static>(input: &Expression<L>) -> Result<TypedExpression<L>> {
    Checker::new(input).check()
}

fn type_eq<L: Loc>(e: &TypedExpression<L>, t: Type) -> bool {
    e.payload.loc_type == t
}

struct Checker<'a, L: Loc> {
    input: &'a Expression<L>,
}

impl<'a, L: Loc + 'static> Checker<'a, L> {
    fn new(input: &'a Expression<L>) -> Self {
        Checker { input }
    }

    fn loc(&self, loc_type: Type) -> TypedLoc<L> {
        TypedLoc::new(self.input, loc_type)
    }

    // Creates and returns an error
    fn error(&mut self, kind: TypeErrorKind) -> Result<TypedExpression<L>> {
        let mut errors: Errors = Default::default();
        errors.add(TypeError::new(self.input, kind));
        Err(errors)
    }

    fn check(&mut self) -> Result<TypedExpression<L>> {
        match &self.input.expr {
            ast::Expr::LitInteger(value) => Ok(Expression::lit_integer(
                self.loc(Type::Integer),
                value.clone(),
            )),
            ast::Expr::Binary(op, e1, e2) => {
                let c1 = Self::new(e1.as_ref()).check();
                let c2 = Self::new(e2.as_ref()).check();
                if c1.is_err() || c2.is_err() {
                    let mut errors: Errors = Default::default();
                    c1.map_err(|e| errors.append(e));
                    c2.map_err(|e| errors.append(e));
                    Err(errors)
                } else {
                    let e1 = c1.unwrap();
                    let e2 = c2.unwrap();
                    match op {
                        ast::BinaryOp::Add => self.check_add(e1, e2),
                        _ => self.error(TypeErrorKind::InvalidType),
                    }
                }
            }
            _ => self.error(TypeErrorKind::InvalidType),
        }
    }

    fn check_add(
        &mut self,
        e1: TypedExpression<L>,
        e2: TypedExpression<L>,
    ) -> Result<TypedExpression<L>> {
        if type_eq(&e1, Type::Integer) && type_eq(&e2, Type::Integer) {
            Ok(Expression::binary(
                self.loc(Type::Integer),
                ast::BinaryOp::Add,
                e1,
                e2,
            ))
        } else {
            self.error(TypeErrorKind::InvalidType)
        }
    }
}

#[cfg(test)]
mod tests;
