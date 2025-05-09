use im::HashMap;

use crate::Integer;
use crate::context::Value;
use crate::error::{Error, L, Loc, Result};
use crate::symbol::Symbol;
use crate::tst::{Block, Expr, Package, TwoInts};

use super::Env;

pub(super) fn run(env: &mut Env, package: &Package) -> Result<Value> {
    Interpreter { env, package }.run()
}

#[derive(Debug, Default, Clone)]
struct Scope {
    values: HashMap<Symbol, Value>,
}

impl Scope {
    fn contains(&self, symbol: &Symbol) -> bool {
        self.values.contains_key(symbol)
    }

    fn get(&self, symbol: &Symbol) -> Option<Value> {
        self.values.get(symbol).cloned()
    }

    pub(crate) fn set(&mut self, loc: &Loc, symbol: Symbol, value: Value) -> Result<()> {
        if self.values.contains_key(&symbol) {
            loc.err(Error::DuplicateLocalSymbol(symbol))
        } else {
            self.values.insert(symbol.clone(), value.clone());
            Ok(())
        }
    }
}

#[derive(Debug)]
struct Interpreter<'a> {
    env: &'a mut Env,
    package: &'a Package,
}

impl<'a> Interpreter<'a> {
    fn run(mut self) -> Result<Value> {
        self.env.types = self.package.types.clone();
        self.env.symbols = self.package.symbols.clone();
        let mut value = Value::v_none();
        let mut scope = Scope::default();
        for a in &self.package.assignments {
            value = self.expression(&mut scope, &a.it.expr)?;
            self.env
                .values
                .set(&a.loc, a.it.symbol.clone(), value.clone())?
        }
        if let Some(e) = &self.package.expr {
            value = self.expression(&mut scope, e)?;
        }
        Ok(value)
    }

    fn block(&self, scope: &mut Scope, block: &Block) -> Result<Value> {
        let mut value = Value::v_none();
        for a in &block.assignments {
            value = self.expression(scope, &a.it.expr)?;
            scope.set(&a.loc, a.it.symbol.clone(), value.clone())?;
        }
        match &block.expr {
            Some(e) => self.expression(scope, e),
            _ => Ok(value),
        }
    }

    fn expression(&self, scope: &mut Scope, expr: &L<Expr>) -> Result<Value> {
        match &expr.it {
            Expr::Value(v) => Ok(v.clone()),
            Expr::Local(l) => match scope.get(&l.symbol) {
                Some(value) => Ok(value),
                None => expr.err(Error::UnknownLocalSymbol(l.symbol.clone())),
            },
            Expr::Global(g) => match self.env.values.get(&g.symbol) {
                Some(value) => Ok(value.clone()),
                None => expr.err(Error::UnknownSymbol(g.symbol.clone())),
            },
            Expr::Seq(s) => self.expression(scope, &s.then), // no effects for now
            Expr::Conditional(c) => {
                if self.expression(scope, &c.expr)?.as_boolean(&expr.loc)? {
                    self.expression(scope, &c.then)
                } else {
                    self.expression(scope, &c.otherwise)
                }
            }
            Expr::IntAdd(t) => self
                .eval_two_ints(&expr.loc, scope, t)
                .map(|(v1, v2)| Value::v_integer(&expr.loc, expr.get_type(), v1 + v2))?,
            Expr::IntSub(t) => self
                .eval_two_ints(&expr.loc, scope, t)
                .map(|(v1, v2)| Value::v_integer(&expr.loc, expr.get_type(), v1 - v2))?,
            Expr::IntMul(t) => self
                .eval_two_ints(&expr.loc, scope, t)
                .map(|(v1, v2)| Value::v_integer(&expr.loc, expr.get_type(), v1 * v2))?,
            Expr::IntDiv(t) => self.div(&expr, scope, t),
            Expr::LogicalAnd(t) => self.and(&expr.loc, scope, &t.expr1, &t.expr2),
            Expr::LogicalOr(t) => self.or(&expr.loc, scope, &t.expr1, &t.expr2),
            Expr::Block(block) => self.block(&mut scope.clone(), block),
            _ => expr.err(Error::NotImplemented),
        }
    }

    fn eval_two_ints(
        &self,
        loc: &Loc,
        scope: &mut Scope,
        t: &TwoInts,
    ) -> Result<(Integer, Integer)> {
        Error::merge(
            self.expression(scope, &t.expr1)?.as_integer(loc),
            self.expression(scope, &t.expr2)?.as_integer(loc),
        )
    }

    fn eval_bool(&self, loc: &Loc, scope: &mut Scope, expr: &L<Expr>) -> Result<bool> {
        self.expression(scope, expr)?.as_boolean(loc)
    }

    fn div(&self, expr: &L<Expr>, scope: &mut Scope, t: &TwoInts) -> Result<Value> {
        let (v1, v2) = self.eval_two_ints(&expr.loc, scope, t)?;
        // We only have integers for now
        if v2.is_zero() {
            expr.err(Error::DivisionByZero)
        } else {
            Value::v_integer(&expr.loc, expr.get_type(), v1 / v2)
        }
    }

    fn and(&self, loc: &Loc, scope: &mut Scope, expr1: &L<Expr>, expr2: &L<Expr>) -> Result<Value> {
        if self.eval_bool(loc, scope, expr1)? {
            Ok(Value::v_bool(self.eval_bool(loc, scope, expr2)?))
        } else {
            Ok(Value::v_false()) // short-circuit
        }
    }

    fn or(&self, loc: &Loc, scope: &mut Scope, expr1: &L<Expr>, expr2: &L<Expr>) -> Result<Value> {
        if self.eval_bool(loc, scope, expr1)? {
            Ok(Value::v_true()) // short-circuit
        } else {
            Ok(Value::v_bool(self.eval_bool(loc, scope, expr2)?))
        }
    }
}

#[cfg(test)]
mod tests;
