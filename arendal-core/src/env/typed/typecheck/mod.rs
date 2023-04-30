use im::HashMap;

use crate::ast::{self, BinaryOp};
use crate::error::{Error, Loc, Result};
use crate::symbol::{FQType, Path, Pkg, Symbol, TSymbol};
use crate::types::Type;
use crate::value::Value;

use crate::env::{Env, TypeCheckError};

use super::{ExprBuilder, Expression, Expressions, Module};

type Scope = HashMap<Symbol, Type>;

pub(super) fn check(env: &Env, path: &Path, input: &ast::Module) -> Result<Module> {
    TypeChecker {
        env,
        path,
        scopes: vec![Scope::default()],
        types: HashMap::default(),
    }
    .module(input)
}

#[derive(Debug)]
struct TypeChecker<'a> {
    env: &'a Env,
    path: &'a Path,
    scopes: Vec<Scope>,
    types: HashMap<TSymbol, Type>,
}

impl<'a> TypeChecker<'a> {
    fn module(&mut self, input: &ast::Module) -> Result<Module> {
        let mut expressions: Vec<Expression> = Vec::default();
        for item in input {
            match item {
                ast::ModuleItem::Expression(e) => {
                    let checked = ExprChecker {
                        checker: self,
                        input: e,
                    }
                    .check()?;
                    expressions.push(checked);
                }
            }
        }
        Ok(Module {
            path: self.path.clone(),
            expressions: Expressions::new(expressions),
        })
    }

    fn set_val(&mut self, loc: Loc, symbol: Symbol, tipo: Type) -> Result<()> {
        self.scopes.last_mut().unwrap().insert(symbol, tipo);
        return Ok(());
    }

    fn get_val(&self, symbol: &Symbol) -> Option<Type> {
        let mut i = self.scopes.len();
        while i > 0 {
            let result = self.scopes[i - 1].get(symbol);
            if result.is_some() {
                return result.cloned();
            }
            i = i - 1;
        }
        if let Some(vv) = self.env.values.get(&&self.path.fq_sym(symbol.clone())) {
            return Some(vv.unwrap().clone_type());
        }
        None
    }

    fn fq_type(&self, symbol: &TSymbol) -> FQType {
        self.path.fq_type(symbol.clone())
    }

    fn resolve_type(&self, loc: &Loc, symbol: &TSymbol) -> Result<Type> {
        match self.types.get(symbol) {
            Some(t) => Ok(t.clone()),
            None => match self.env.types.get(&self.fq_type(symbol)) {
                Some(t) => Ok(t.cloned()),
                None => match self
                    .env
                    .types
                    .get(&Pkg::Std.empty().fq_type(symbol.clone()))
                {
                    Some(t) => Ok(t.cloned()),
                    None => loc.err(TypeCheckError::UnknownType(symbol.clone())),
                },
            },
        }
    }
}

#[derive(Debug)]
struct ExprChecker<'a, 'b> {
    checker: &'b mut TypeChecker<'a>,
    input: &'b ast::Expression,
}

impl<'a, 'b> ExprChecker<'a, 'b> {
    fn check(mut self) -> Result<Expression> {
        match &self.input.expr {
            ast::Expr::LitInteger(value) => Ok(self.builder().val_integer(value.clone())),
            ast::Expr::Symbol(id) => match self.checker.get_val(&id) {
                Some(tipo) => Ok(self.builder().local(id.clone(), tipo.clone())),
                None => self.error(TypeCheckError::UnknownIdentifier(id.clone())),
            },
            ast::Expr::TSymbol(s) => {
                let tipo = self.resolve_type(&s)?;
                let value = Value::singleton(&self.input.loc, &tipo)?;
                Ok(self.builder().value(value))
            }
            ast::Expr::Assignment(a) => {
                let typed = self.sub_expr(&a.expr)?;
                self.checker.set_val(
                    self.input.loc.clone(),
                    a.symbol.clone(),
                    typed.clone_type(),
                )?;
                Ok(self.builder().assignment(a.symbol.clone(), typed))
            }
            ast::Expr::Binary(b) => Error::merge(
                self.sub_expr(&b.expr1),
                self.sub_expr(&b.expr2),
                |t1, t2| self.check_binary(b.op, t1, t2),
            ),
            _ => self.error(TypeCheckError::InvalidType),
        }
    }

    fn resolve_type(&self, symbol: &TSymbol) -> Result<Type> {
        self.checker.resolve_type(&self.input.loc, symbol)
    }

    fn sub_expr(&mut self, input: &ast::Expression) -> Result<Expression> {
        ExprChecker {
            checker: self.checker,
            input,
        }
        .check()
    }

    fn check_binary(self, op: BinaryOp, e1: Expression, e2: Expression) -> Result<Expression> {
        match op {
            BinaryOp::Add => self.check_add(e1, e2),
            BinaryOp::Sub => self.check_sub(e1, e2),
            BinaryOp::Mul => self.check_mul(e1, e2),
            BinaryOp::Div => self.check_div(e1, e2),
            _ => self.error(TypeCheckError::InvalidType),
        }
    }

    fn check_add(self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        if expr1.borrow_type().is_integer() && expr2.borrow_type().is_integer() {
            Ok(self.builder().add(expr1, expr2))
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn check_sub(self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        if expr1.borrow_type().is_integer() && expr2.borrow_type().is_integer() {
            Ok(self.builder().sub(expr1, expr2))
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn check_mul(self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        if expr1.borrow_type().is_integer() && expr2.borrow_type().is_integer() {
            Ok(self.builder().mul(expr1, expr2))
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn check_div(self, expr1: Expression, expr2: Expression) -> Result<Expression> {
        if expr1.borrow_type().is_integer() && expr2.borrow_type().is_integer() {
            Ok(self.builder().div(expr1, expr2))
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn builder(&self) -> ExprBuilder {
        ExprBuilder::new(self.input.loc.clone())
    }

    // Creates and returns an error
    fn error(self, error: TypeCheckError) -> Result<Expression> {
        self.input.loc.err(error)
    }
}

#[cfg(test)]
mod tests;
