mod expr;
mod module;
mod types;

use std::collections::HashMap;

use crate::ast::{self, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQType, Pkg, TSymbol};
use crate::types::{Type, Types};

use crate::env::{Env, Symbols};
use crate::visibility::V;

use self::module::Module;
use super::{Expr, ExprBuilder, Package, Stmt, TLAssignment, TLStmt, Value};

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<Package> {
    let input = Input {
        env,
        pkg: input.pkg.clone(),
        modules: module::get_modules(input)?,
    };
    let types = types::check(&input)?;
    PackageChecker {
        input,
        types,
        symbols: env.symbols.clone(),
    }
    .check()
}

struct TCandidate<'a> {
    dfn: &'a ast::TypeDefinition,
}

type TCandidates<'a> = HashMap<FQType, TCandidate<'a>>;

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
    symbols: Symbols,
}

impl<'a> PackageChecker<'a> {
    fn new(env: &Env, input: &ast::Package) {
        let mut errors = Errors::default();
        let mut t_candidates = TCandidates::default();
        for (path, module) in &input.modules {
            let fq_path = input.pkg.path(path.clone());
            for dfn in &module.types {
                let fq_type = fq_path.fq_type(dfn.symbol.clone());
                if t_candidates.contains_key(&fq_type) {
                    errors.add(dfn.loc.wrap(Error::DuplicateType(fq_type)));
                } else {
                    t_candidates.insert(fq_type, TCandidate { dfn });
                }
            }
        }
    }
    fn check(mut self) -> Result<Package> {
        let mut statements = Vec::default();
        let mut errors = Errors::default();
        for module in &self.input.modules {
            let mut checker = ModuleChecker::new(&self, module);
            errors.add_result(checker.check());
            statements.append(&mut checker.statements);
        }
        errors.to_lazy_result(|| Package {
            pkg: self.input.pkg.clone(),
            types: self.types,
            statements,
        })
    }
}

type Scope = crate::env::Scope<Type>;

#[derive(Debug)]
struct ModuleChecker<'a> {
    pkg: &'a PackageChecker<'a>,
    input: &'a Module<'a>,
    scope: Scope,
    statements: Vec<L<TLStmt>>,
}

impl<'a> ModuleChecker<'a> {
    fn new(pkg: &'a PackageChecker, input: &'a Module) -> ModuleChecker<'a> {
        ModuleChecker {
            pkg,
            input,
            scope: Scope::default(),
            statements: Vec::default(),
        }
    }

    fn check(&mut self) -> Result<()> {
        for a in &self.input.ast.assignments {
            let checked = self.check_assignment(a)?;
            self.statements.push(checked)
        }
        for e in &self.input.ast.exprs {
            let checked = self.check_expression(e)?;
            self.statements.push(checked)
        }
        Ok(())
    }

    fn check_assignment(&mut self, a: &L<V<ast::Assignment>>) -> Result<L<TLStmt>> {
        let symbol = a.it.it.symbol.clone();
        if self.scope.contains(&symbol) {
            a.loc
                .err(Error::DuplicateSymbol(self.input.path.fq_sym(symbol)))
        } else {
            let expr = expr::check(self, &self.scope, &a.it.it.expr)?;
            self.scope.set(&a.loc, symbol.clone(), expr.clone_type())?;
            Ok(a.loc.wrap(
                TLAssignment {
                    symbol: self.input.path.fq_sym(symbol),
                    expr,
                }
                .to_stmt(),
            ))
        }
    }

    fn check_expression(&mut self, e: &L<ast::Expr>) -> Result<L<TLStmt>> {
        Ok(expr::check(self, &self.scope, e)?.to_tl_stmt())
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
