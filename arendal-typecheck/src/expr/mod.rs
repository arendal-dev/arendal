use super::TypeError;
use ast::{BinaryOp, Errors, Expr, Expression, Loc, Result, Type, TypedExpr};

// 'static here means that L is owned
pub(crate) fn check(input: Expression) -> Result<TypedExpr> {
    Checker::new(input).check()
}

fn type_eq(e: &TypedExpr, t: Type) -> bool {
    *e.borrow_type() == t
}

struct Checker {
    input: Expression,
}

impl Checker {
    fn new(input: Expression) -> Self {
        Checker { input }
    }

    fn loc(&self) -> Loc {
        self.input.borrow_loc().clone()
    }

    // Creates and returns an error
    fn error(self, kind: TypeError) -> Result<TypedExpr> {
        let mut errors: Errors = Default::default();
        errors.add(self.loc(), kind);
        Err(errors)
    }

    fn check(self) -> Result<TypedExpr> {
        match self.input.borrow_expr() {
            Expr::LitInteger(value) => Ok(TypedExpr::lit_integer(self.loc(), value.clone())),
            Expr::Binary(op, e1, e2) => {
                let c1 = Self::new(e1.clone()).check();
                let c2 = Self::new(e2.clone()).check();
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
                        ast::BinaryOp::Sub => self.check_sub(e1, e2),
                        ast::BinaryOp::Mul => self.check_mul(e1, e2),
                        ast::BinaryOp::Div => self.check_div(e1, e2),
                        _ => self.error(TypeError::InvalidType),
                    }
                }
            }
            _ => self.error(TypeError::InvalidType),
        }
    }

    fn ok_binary(
        &self,
        expr_type: Type,
        op: BinaryOp,
        e1: TypedExpr,
        e2: TypedExpr,
    ) -> Result<TypedExpr> {
        Ok(TypedExpr::binary(self.loc(), expr_type, op, e1, e2))
    }

    fn check_add(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if type_eq(&e1, Type::Integer) && type_eq(&e2, Type::Integer) {
            self.ok_binary(Type::Integer, BinaryOp::Add, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_sub(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if type_eq(&e1, Type::Integer) && type_eq(&e2, Type::Integer) {
            self.ok_binary(Type::Integer, BinaryOp::Sub, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_mul(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if type_eq(&e1, Type::Integer) && type_eq(&e2, Type::Integer) {
            self.ok_binary(Type::Integer, BinaryOp::Mul, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }

    fn check_div(self, e1: TypedExpr, e2: TypedExpr) -> Result<TypedExpr> {
        if type_eq(&e1, Type::Integer) && type_eq(&e2, Type::Integer) {
            self.ok_binary(Type::Integer, BinaryOp::Div, e1, e2)
        } else {
            self.error(TypeError::InvalidType)
        }
    }
}

#[cfg(test)]
mod tests;
