mod expr;
mod module;
mod types;

use im::HashMap;

use crate::ast::{self, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, Pkg, Symbol, TSymbol};
use crate::types::{Type, Types};

use crate::env::Env;

use self::module::Module;
use super::{Assignment, Expr, ExprBuilder, Package, Stmt, Value};

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

#[derive(Debug)]
struct ModuleChecker<'a> {
    pkg: &'a PackageChecker<'a>,
    input: &'a Module<'a>,
    scopes: Vec<Scope>,
    statements: Vec<L<Stmt>>,
}

impl<'a> ModuleChecker<'a> {
    fn new(pkg: &'a PackageChecker, input: &'a Module) -> ModuleChecker<'a> {
        ModuleChecker {
            pkg,
            input,
            scopes: vec![Scope::default()],
            statements: Vec::default(),
        }
    }

    fn check(&mut self) -> Result<()> {
        for s in &self.input.ast.statements {
            let checked = match &s.it {
                ast::Stmt::Assignment(a) => self.check_assignment(&s.loc, a.as_ref())?,
                ast::Stmt::Expr(e) => self.check_expression(&s.loc, e.as_ref())?,
            };
            self.statements.push(checked)
        }
        Ok(())
    }

    fn check_assignment(&mut self, loc: &Loc, a: &ast::Assignment) -> Result<L<Stmt>> {
        let symbol = a.symbol.clone();
        if self.scopes.last().unwrap().contains_key(&symbol) {
            loc.err(Error::DuplicateSymbol(self.input.path.fq_sym(symbol)))
        } else {
            let expr = expr::check(self, &a.expr)?;
            self.set_val(loc.clone(), symbol.clone(), expr.clone_type())?;
            Ok(loc.wrap(Assignment { symbol, expr }.to_stmt()))
        }
    }

    fn check_expression(&mut self, loc: &Loc, e: &L<ast::Expr>) -> Result<L<Stmt>> {
        Ok(expr::check(self, e)?.to_stmt())
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
            .get(&self.input.path.fq_sym(symbol.clone()))
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
