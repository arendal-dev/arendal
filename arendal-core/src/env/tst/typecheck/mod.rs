use im::HashMap;

use crate::ast::{self, BinaryOp};
use crate::error::{Error, Loc, Result};
use crate::symbol::{FQType, Path, Pkg, Symbol, TSymbol};
use crate::types::{Type, Types};

use crate::env::Env;
use crate::visibility::Visibility;

use super::{ExprBuilder, Expression, Expressions, Module, TypeDefinition, TypeDefinitions, Value};

type Scope = HashMap<Symbol, Type>;

pub(super) fn check(env: &Env, path: &Path, input: &ast::Module) -> Result<Module> {
    TypeChecker {
        env,
        path,
        scopes: vec![Scope::default()],
        types: LocalTypes::default(),
    }
    .module(input)
}

#[derive(Debug, Default)]
struct LocalTypes {
    types: HashMap<TSymbol, Type>,
}

impl LocalTypes {
    fn get(&self, symbol: &TSymbol) -> Option<&Type> {
        self.types.get(symbol)
    }

    fn check_available(&self, dfn: &ast::TypeDefinition) -> Result<()> {
        if self.types.contains_key(&dfn.symbol) {
            dfn.loc.err(Error::DuplicateLocalType(dfn.symbol.clone()))
        } else {
            Ok(())
        }
    }

    fn insert_complete(&mut self, dfn: &ast::TypeDefinition, tipo: Type) -> Result<()> {
        self.check_available(dfn)?;
        self.types.insert(dfn.symbol.clone(), tipo);
        Ok(())
    }
}

#[derive(Debug)]
struct TypeChecker<'a> {
    env: &'a Env,
    path: &'a Path,
    scopes: Vec<Scope>,
    types: LocalTypes,
}

impl<'a> TypeChecker<'a> {
    fn module(&mut self, input: &ast::Module) -> Result<Module> {
        Ok(Module {
            path: self.path.clone(),
            types: self.check_types(input)?,
            expressions: self.check_expressions(input)?,
        })
    }

    fn check_types(&mut self, input: &ast::Module) -> Result<TypeDefinitions> {
        let mut types: Vec<TypeDefinition> = Vec::default();
        for item in input {
            if let ast::ModuleItem::TypeDefinition(t) = item {
                match t.dfn {
                    ast::TypeDfn::Singleton => self.types.insert_complete(
                        t,
                        self.env
                            .types
                            .singleton(&t.loc, self.path.fq_type(t.symbol.clone()))?,
                    )?,
                }
            }
        }
        Ok(TypeDefinitions::new(types))
    }

    fn check_expressions(&mut self, input: &ast::Module) -> Result<Expressions> {
        let mut expressions: Vec<Expression> = Vec::default();
        for item in input {
            if let ast::ModuleItem::Expression(e) = item {
                let checked = ExprChecker {
                    checker: self,
                    input: e,
                }
                .check()?;
                expressions.push(checked);
            }
        }
        Ok(Expressions::new(expressions))
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
            None => self
                .env
                .types
                .get(&self.fq_type(symbol))
                .or_else(|| {
                    self.env
                        .types
                        .get(&Pkg::Std.empty().fq_type(symbol.clone()))
                })
                .map_or_else(
                    || loc.err(Error::UnknownLocalType(symbol.clone())),
                    |t| Ok(t.cloned()),
                ),
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
                None => self.error(Error::UnknownLocalSymbol(id.clone())),
            },
            ast::Expr::TSymbol(s) => {
                let tipo = self.resolve_type(&s)?;
                let value = Value::singleton(&self.input.loc, &tipo)?;
                Ok(self.builder().value(value))
            }
            ast::Expr::Conditional(c) => {
                let (expr, then, otherwise) = Error::merge3(
                    self.sub_expr(&c.expr),
                    self.sub_expr(&c.then),
                    self.sub_expr(&c.otherwise),
                )?;
                self.builder().conditional(expr, then, otherwise)
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
            ast::Expr::Binary(b) => Error::merge(self.sub_expr(&b.expr1), self.sub_expr(&b.expr2))
                .and_then(|(t1, t2)| self.check_binary(b.op, t1, t2)),
            ast::Expr::Block(v) => {
                self.checker.scopes.push(Scope::default());
                let result = self.check_block(v);
                self.checker.scopes.pop();
                result
            }
            _ => self.error(Error::InvalidType),
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

    fn check_binary(
        self,
        op: BinaryOp,
        expr1: Expression,
        expr2: Expression,
    ) -> Result<Expression> {
        match op {
            BinaryOp::Add => self.builder().int_add(expr1, expr2),
            BinaryOp::Sub => self.builder().int_sub(expr1, expr2),
            BinaryOp::Mul => self.builder().int_mul(expr1, expr2),
            BinaryOp::Div => self.builder().int_div(expr1, expr2),
            BinaryOp::And => self.builder().log_and(expr1, expr2),
            BinaryOp::Or => self.builder().log_or(expr1, expr2),
            _ => self.error(Error::InvalidType),
        }
    }

    fn check_block(&mut self, exprs: &Vec<ast::Expression>) -> Result<Expression> {
        let mut checked = Vec::default();
        for e in exprs {
            checked.push(self.sub_expr(e)?);
        }
        self.builder().block(checked)
    }

    fn builder(&self) -> ExprBuilder {
        ExprBuilder::new(self.input.loc.clone())
    }

    // Creates and returns an error
    fn error(self, error: Error) -> Result<Expression> {
        self.input.loc.err(error)
    }
}

#[cfg(test)]
mod tests;
