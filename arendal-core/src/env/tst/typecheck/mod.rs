mod expr;

use im::HashMap;

use crate::ast;
use crate::error::{Error, Errors, Loc, Result};
use crate::symbol::{FQPath, FQType, Pkg, Symbol, TSymbol};
use crate::types::Type;

use crate::env::Env;
use crate::visibility::Visibility;

use super::{ExprBuilder, Expression, Package, TypeDefMap, TypeDefinition, Value};

type Scope = HashMap<Symbol, Type>;

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<Package> {
    let types = TypesChecker {
        env,
        input,
        types: TypeDefMap::default(),
        errors: Errors::default(),
    }
    .check()?;
    PackageChecker { env, input, types }.check()
}

#[derive(Debug)]
struct TypesChecker<'a> {
    env: &'a Env,
    input: &'a ast::Package,
    types: TypeDefMap,
    errors: Errors,
}

impl<'a> TypesChecker<'a> {
    fn check(mut self) -> Result<TypeDefMap> {
        for (path, module) in &self.input.modules {
            self.check_module(self.input.pkg.path(path.clone()), module)
        }
        Ok(self.types)
    }

    fn check_module(&mut self, path: FQPath, module: &ast::Module) {
        for t in &module.types {
            let fq = path.fq_type(t.symbol.clone());
            let maybe = if self.types.contains_key(&fq) || self.env.types.contains(&fq) {
                self.errors
                    .add(t.loc.wrap(Error::DuplicateType(fq.clone())));
                None
            } else {
                match t.dfn {
                    ast::TypeDfn::Singleton => {
                        self.errors.add_result(Type::singleton(&t.loc, fq.clone()))
                    }
                }
            };
            if let Some(tipo) = maybe {
                self.types.insert(
                    fq,
                    TypeDefinition {
                        loc: t.loc.clone(),
                        visibility: Visibility::Module,
                        tipo,
                    },
                );
            }
        }
    }
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
    fn check(mut self) -> Result<Vec<Expression>> {
        self.check_expressions()
    }

    fn check_expressions(&mut self) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = Vec::default();
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
