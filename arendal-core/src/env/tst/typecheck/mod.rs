mod expr;
mod types;

use im::HashMap;

use crate::ast;
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, FQType, Pkg, Symbol, TSymbol};
use crate::types::Type;

use crate::env::Env;

use super::{Expr, ExprBuilder, Package, TypeDefMap, TypeDefinition, Value};

type Scope = HashMap<Symbol, Type>;

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<Package> {
    let types = types::check(env, input)?;
    PackageChecker { env, input, types }.check()
}

#[derive(Debug)]
struct PackageChecker<'a> {
    env: &'a Env,
    input: &'a ast::Package,
    types: TypeDefMap,
}

impl<'a> PackageChecker<'a> {
    fn check(mut self) -> Result<Package> {
        let mut expressions = Vec::default();
        let mut errors = Errors::default();
        for (path, module) in &self.input.modules {
            let fqpath = self.input.pkg.path(path.clone());
            errors
                .add_result(
                    ModuleChecker {
                        pkg: &mut self,
                        path: fqpath,
                        input: module,
                        scopes: vec![Scope::default()],
                    }
                    .check(),
                )
                .map(|mut e| expressions.append(&mut e));
        }
        errors.to_lazy_result(|| Package {
            pkg: self.input.pkg.clone(),
            types: self.types,
            expressions,
        })
    }
}

#[derive(Debug)]
struct ModuleChecker<'a> {
    pkg: &'a PackageChecker<'a>,
    input: &'a ast::Module,
    path: FQPath,
    scopes: Vec<Scope>,
}

impl<'a> ModuleChecker<'a> {
    fn check(mut self) -> Result<Vec<L<Expr>>> {
        self.check_expressions()
    }

    fn check_expressions(&mut self) -> Result<Vec<L<Expr>>> {
        let mut expressions: Vec<L<Expr>> = Vec::default();
        for e in &self.input.expressions {
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
        if let Some(vv) = self.pkg.env.values.get(&self.path.fq_sym(symbol.clone())) {
            return Some(vv.unwrap().clone_type());
        }
        None
    }

    fn fq_type(&self, symbol: &TSymbol) -> FQType {
        self.path.fq_type(symbol.clone())
    }

    fn resolve_type(&self, loc: &Loc, symbol: &TSymbol) -> Result<Type> {
        let fq = self.path.fq_type(symbol.clone());
        match self.pkg.types.get(&fq) {
            Some(t) => Ok(t.tipo.clone()),
            None => self
                .pkg
                .env
                .types
                .get(&self.fq_type(symbol))
                .or_else(|| {
                    self.pkg
                        .env
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
