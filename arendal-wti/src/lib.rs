use ast::{Loc, TypedExpression};
use num::Integer;

pub enum Value {
    Integer(Integer),
}

pub struct RuntimeError<L: Loc> {
    loc: L,
}

pub fn expression<L: Loc>(expr: &TypedExpression<L>) -> Result<Value, RuntimeError<L>> {
    match &expr.expr {
        ast::Expr::LitInteger(i) => Ok(Value::Integer(i.clone())),
        _ => Err(RuntimeError {
            loc: expr.payload.loc.clone(),
        }),
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
