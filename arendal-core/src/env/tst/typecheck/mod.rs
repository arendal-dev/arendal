mod expr;
mod module;
mod types;

use crate::ast::{self, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{Pkg, TSymbol};
use crate::types::{Type, Types};

use crate::env::{Env, Symbols};

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
        for s in &self.input.ast.statements {
            let checked = match &s.it {
                ast::BStmt::Assignment(a) => self.check_assignment(&s.loc, a.as_ref())?,
                ast::BStmt::Expr(e) => self.check_expression(e.as_ref())?,
            };
            self.statements.push(checked)
        }
        Ok(())
    }

    fn check_assignment(&mut self, loc: &Loc, a: &ast::Assignment) -> Result<L<TLStmt>> {
        let symbol = a.symbol.clone();
        if self.scope.contains(&symbol) {
            loc.err(Error::DuplicateSymbol(self.input.path.fq_sym(symbol)))
        } else {
            let expr = expr::check(self, &self.scope, &a.expr)?;
            self.scope.set(&loc, symbol.clone(), expr.clone_type())?;
            Ok(loc.wrap(
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
