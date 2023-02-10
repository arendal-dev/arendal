use ast::{Loc, Type, TypedExpression};
use num::Integer;

pub enum Value {
    Integer(Integer),
}

pub struct TypedValue {
    pub value: Value,
    pub value_type: Type,
}

impl TypedValue {
    fn new(value: Value, value_type: Type) -> Self {
        TypedValue { value, value_type }
    }
}

pub struct RuntimeError<L: Loc> {
    loc: L,
}

fn ok<L: Loc>(value: Value, value_type: Type) -> Result<TypedValue, RuntimeError<L>> {
    Ok(TypedValue::new(value, value_type))
}

pub fn expression<L: Loc>(expr: &TypedExpression<L>) -> Result<TypedValue, RuntimeError<L>> {
    match &expr.expr {
        ast::Expr::LitInteger(i) => ok(Value::Integer(i.clone()), expr.payload.loc_type.clone()),
        _ => Err(RuntimeError {
            loc: expr.payload.loc.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
