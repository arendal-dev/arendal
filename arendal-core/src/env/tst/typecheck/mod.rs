mod expr;
mod module;
mod types;

use im::HashMap;

use crate::ast::{self, Q};
use crate::error::{Errors, Loc, Result, L};
use crate::symbol::{FQPath, Pkg, Symbol, TSymbol};
use crate::types::{Type, Types};

use crate::env::Env;

use self::module::Module;
use super::{Expr, ExprBuilder, Package, Value};

type Scope = HashMap<Symbol, Type>;

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<Package> {
    let input = Input {
        env,
        pkg: input.pkg.clone(),
        modules: module::get_modules(input)?,
    };
    let types = types::check(&input)?;
    PackageChecker { input, types }.check()
}

#[derive(Debug)]
struct Input<'a> {
    env: &'a Env,
    pkg: Pkg,
    modules: Vec<Module<'a>>,
}

#[derive(Debug)]
struct PackageChecker<'a> {
    input: Input<'a>,
    types: Types,
}

impl<'a> PackageChecker<'a> {
    fn check(mut self) -> Result<Package> {
        let mut expressions = Vec::default();
        let mut errors = Errors::default();
        for module in &self.input.modules {
            errors
                .add_result(
                    ModuleChecker {
                        pkg: &self,
                        path: module.path.clone(),
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
    input: &'a Module<'a>,
    path: FQPath,
    scopes: Vec<Scope>,
}

impl<'a> ModuleChecker<'a> {
    fn check(mut self) -> Result<Vec<L<Expr>>> {
        self.check_expressions()
    }

    fn check_expressions(&mut self) -> Result<Vec<L<Expr>>> {
        let mut expressions: Vec<L<Expr>> = Vec::default();
        for e in &self.input.ast.expressions {
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
        if let Some(vv) = self
            .pkg
            .input
            .env
            .values
            .get(&self.path.fq_sym(symbol.clone()))
        {
            return Some(vv.it.clone_type());
        }
        None
    }

    fn resolve_type(&self, loc: &Loc, symbol: &Q<TSymbol>) -> Result<Type> {
        let fq = self
            .input
            .resolve_type(|fq| self.pkg.types.contains(fq), loc, symbol)?;
        Ok(self.pkg.types.get(&fq).unwrap().it.clone())
    }
}

#[cfg(test)]
mod tests;
