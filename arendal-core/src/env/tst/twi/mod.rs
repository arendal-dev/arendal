use super::{BStmt, Env, Expr, Package, TwoInts, Value};
use crate::error::{Error, Loc, Result, L};
use crate::symbol::{FQPath, Symbol};
use crate::visibility::Visibility;
use crate::Integer;
use std::collections::HashMap;

type Scope = HashMap<Symbol, Value>;

pub(super) fn interpret(env: &mut Env, package: &Package) -> Result<Value> {
    Interpreter {
        env,
        package: package,
        path: package.pkg.empty(),
        scopes: Default::default(),
    }
    .run()
}

#[derive(Debug)]
struct Interpreter<'a> {
    env: &'a mut Env,
    package: &'a Package,
    path: FQPath,
    scopes: Vec<Scope>,
}

impl<'a> Interpreter<'a> {
    fn set_val(&mut self, loc: &Loc, symbol: Symbol, value: Value) -> Result<()> {
        if !self.scopes.is_empty() {
            self.scopes.last_mut().unwrap().insert(symbol, value);
            Ok(())
        } else {
            self.env
                .values
                .set(loc, self.path.fq_sym(symbol), Visibility::Module, value)
        }
    }

    fn get_val(&self, symbol: &Symbol) -> Option<Value> {
        let mut i = self.scopes.len();
        while i > 0 {
            let result = self.scopes[i - 1].get(symbol);
            if result.is_some() {
                return result.cloned();
            }
            i = i - 1;
        }
        if let Some(vv) = self.env.values.get(&self.path.fq_sym(symbol.clone())) {
            return Some(vv.it);
        }
        None
    }

    fn run(mut self) -> Result<Value> {
        let mut value = Value::None;
        for a in &self.package.assignments {
            value = self.expression(&a.it.expr)?;
            self.set_val(&a.loc, a.it.symbol.symbol(), value.clone())?;
        }
        for e in &self.package.exprs {
            value = self.expression(e)?;
        }
        Ok(value)
    }

    fn b_stmts(&mut self, exprs: &Vec<L<BStmt>>) -> Result<Value> {
        let mut value = Value::None;
        for stmt in exprs {
            value = self.b_stmt(stmt)?;
        }
        Ok(value)
    }

    fn b_stmt(&mut self, expr: &L<BStmt>) -> Result<Value> {
        match &expr.it {
            BStmt::Assignment(a) => {
                let value = self.expression(&a.expr)?;
                self.set_val(&expr.loc, a.symbol.clone(), value.clone())?;
                Ok(value)
            }
            BStmt::Expr(t) => self.expression(t.as_ref()),
        }
    }

    fn expression(&mut self, expr: &L<Expr>) -> Result<Value> {
        match &expr.it {
            Expr::Value(v) => Ok(v.clone()),
            Expr::Local(l) => match self.get_val(&l.symbol) {
                Some(value) => Ok(value),
                None => expr.err(Error::UnknownLocalSymbol(l.symbol.clone())),
            },
            Expr::Conditional(c) => {
                if self.expression(&c.expr)?.as_boolean(&expr.loc)? {
                    self.expression(&c.then)
                } else {
                    self.expression(&c.otherwise)
                }
            }
            Expr::IntAdd(t) => self
                .eval_two_ints(&expr.loc, t)
                .map(|(v1, v2)| Value::Integer(v1 + v2)),
            Expr::IntSub(t) => self
                .eval_two_ints(&expr.loc, t)
                .map(|(v1, v2)| Value::Integer(v1 - v2)),
            Expr::IntMul(t) => self
                .eval_two_ints(&expr.loc, t)
                .map(|(v1, v2)| Value::Integer(v1 * v2)),
            Expr::IntDiv(t) => self.div(&expr.loc, t),
            Expr::LogicalAnd(t) => self.and(&expr.loc, &t.expr1, &t.expr2),
            Expr::LogicalOr(t) => self.or(&expr.loc, &t.expr1, &t.expr2),
            Expr::Block(stmts) => {
                self.scopes.push(Scope::default());
                let value = self.b_stmts(stmts);
                self.scopes.pop();
                value
            }
            _ => expr.err(Error::NotImplemented),
        }
    }

    fn eval_two_ints(&mut self, loc: &Loc, t: &TwoInts) -> Result<(Integer, Integer)> {
        Error::merge(
            self.expression(&t.expr1)?.as_integer(loc),
            self.expression(&t.expr2)?.as_integer(loc),
        )
    }

    fn eval_bool(&mut self, loc: &Loc, expr: &L<Expr>) -> Result<bool> {
        self.expression(expr)?.as_boolean(loc)
    }

    fn div(&mut self, loc: &Loc, t: &TwoInts) -> Result<Value> {
        let (v1, v2) = self.eval_two_ints(loc, t)?;
        // We only have integers for now
        if v2.is_zero() {
            loc.err(Error::DivisionByZero)
        } else {
            Ok(Value::Integer(v1 / v2))
        }
    }

    fn and(&mut self, loc: &Loc, expr1: &L<Expr>, expr2: &L<Expr>) -> Result<Value> {
        if self.eval_bool(loc, expr1)? {
            Ok(Value::boolean(self.eval_bool(loc, expr2)?))
        } else {
            Ok(Value::False) // short-circuit
        }
    }

    fn or(&mut self, loc: &Loc, expr1: &L<Expr>, expr2: &L<Expr>) -> Result<Value> {
        if self.eval_bool(loc, expr1)? {
            Ok(Value::True) // short-circuit
        } else {
            Ok(Value::boolean(self.eval_bool(loc, expr2)?))
        }
    }
}

#[cfg(test)]
mod tests;
