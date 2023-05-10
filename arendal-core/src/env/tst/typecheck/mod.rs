mod expr;

use im::HashMap;

use crate::ast;
use crate::error::{Error, Loc, Result};
use crate::symbol::{FQType, Path, Pkg, Symbol, TSymbol};
use crate::types::Type;

use crate::env::Env;

use super::{ExprBuilder, Expression, Module, TypeDefinition, Value};

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

    fn check_types(&mut self, input: &ast::Module) -> Result<Vec<TypeDefinition>> {
        let mut types: Vec<TypeDefinition> = Vec::default();
        for t in &input.types {
            match t.dfn {
                ast::TypeDfn::Singleton => self.types.insert_complete(
                    t,
                    self.env
                        .types
                        .singleton(&t.loc, self.path.fq_type(t.symbol.clone()))?,
                )?,
            }
        }
        Ok(types)
    }

    fn check_expressions(&mut self, input: &ast::Module) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = Vec::default();
        for e in &input.expressions {
            expressions.push(expr::check(self, e)?);
        }
        Ok(expressions)
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

#[cfg(test)]
mod tests;
