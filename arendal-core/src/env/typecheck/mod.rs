use im::HashMap;

use crate::ast::{self, BinaryOp};
use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::{FQType, Path, Pkg, Symbol, TSymbol};
use crate::typed;
use crate::types::Type;
use crate::value::Value;

use super::Env;

type Scope = HashMap<Symbol, Type>;

pub(super) fn check(env: &Env, path: &Path, input: &ast::Module) -> Result<typed::Module> {
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
    fn module(&mut self, input: &ast::Module) -> Result<typed::Module> {
        let mut expressions: Vec<typed::Expression> = Vec::default();
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
        Ok(typed::Module {
            path: self.path.clone(),
            expressions: typed::Expressions::new(expressions),
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
                    None => Errors::err(loc.clone(), TypeCheckError::UnknownType(symbol.clone())),
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
    fn check(mut self) -> Result<typed::Expression> {
        match self.input.borrow_expr() {
            ast::Expr::LitInteger(value) => Ok(self.builder().val_integer(value.clone())),
            ast::Expr::Symbol(id) => match self.checker.get_val(id) {
                Some(tipo) => Ok(self.builder().val(id.clone(), tipo.clone())),
                None => self.error(TypeCheckError::UnknownIdentifier(id.clone())),
            },
            ast::Expr::TSymbol(s) => {
                let tipo = self.resolve_type(s)?;
                let value = Value::singleton(self.input.borrow_loc(), &tipo)?;
                Ok(self.builder().value(value))
            }
            ast::Expr::Assignment(id, expr) => {
                let typed = self.sub_expr(&expr)?;
                self.checker
                    .set_val(self.input.clone_loc(), id.clone(), typed.clone_type())?;
                Ok(self.builder().assignment(id.clone(), typed))
            }
            ast::Expr::Binary(op, e1, e2) => {
                Errors::merge(self.sub_expr(&e1), self.sub_expr(&e2), |t1, t2| {
                    self.check_binary(*op, t1, t2)
                })
            }
            _ => self.error(TypeCheckError::InvalidType),
        }
    }

    fn resolve_type(&self, symbol: &TSymbol) -> Result<Type> {
        self.checker.resolve_type(self.input.borrow_loc(), symbol)
    }

    fn sub_expr(&mut self, input: &ast::Expression) -> Result<typed::Expression> {
        ExprChecker {
            checker: self.checker,
            input,
        }
        .check()
    }

    fn check_binary(
        self,
        op: BinaryOp,
        e1: typed::Expression,
        e2: typed::Expression,
    ) -> Result<typed::Expression> {
        match op {
            BinaryOp::Add => self.check_add(e1, e2),
            BinaryOp::Sub => self.check_sub(e1, e2),
            BinaryOp::Mul => self.check_mul(e1, e2),
            BinaryOp::Div => self.check_div(e1, e2),
            _ => self.error(TypeCheckError::InvalidType),
        }
    }

    fn ok_binary(
        &self,
        tipo: Type,
        op: BinaryOp,
        e1: typed::Expression,
        e2: typed::Expression,
    ) -> Result<typed::Expression> {
        Ok(self.builder().binary(tipo, op, e1, e2))
    }

    fn check_add(self, e1: typed::Expression, e2: typed::Expression) -> Result<typed::Expression> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::Integer, BinaryOp::Add, e1, e2)
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn check_sub(self, e1: typed::Expression, e2: typed::Expression) -> Result<typed::Expression> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::Integer, BinaryOp::Sub, e1, e2)
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn check_mul(self, e1: typed::Expression, e2: typed::Expression) -> Result<typed::Expression> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::Integer, BinaryOp::Mul, e1, e2)
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn check_div(self, e1: typed::Expression, e2: typed::Expression) -> Result<typed::Expression> {
        if e1.is_integer() && e2.is_integer() {
            self.ok_binary(Type::Integer, BinaryOp::Div, e1, e2)
        } else {
            self.error(TypeCheckError::InvalidType)
        }
    }

    fn builder(&self) -> typed::ExprBuilder {
        typed::ExprBuilder::new(self.input.clone_loc())
    }

    // Creates and returns an error
    fn error(self, kind: TypeCheckError) -> Result<typed::Expression> {
        Errors::err(self.input.clone_loc(), kind)
    }
}

#[derive(Debug)]
enum TypeCheckError {
    UnknownType(TSymbol),
    UnknownIdentifier(Symbol),
    InvalidType, // placeholder, temporary error
}

impl Error for TypeCheckError {}

#[cfg(test)]
mod tests;
