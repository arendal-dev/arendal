mod expr;
mod fqresolver;
mod types;

use std::sync::Arc;

use im::HashMap;

use crate::ast::{self, ExprRef, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol};
use crate::types::{Type, Types};

use crate::env::{Env, Symbols};

use self::fqresolver::FQResolvers;

use super::{Builder, Expr, Global, Local, Package, TLAssignment, Value};

pub(super) fn check(env: &Env, ast: &ast::Package) -> Result<Package> {
    let input = Input::new(env, ast)?;
    let fqresolvers = fqresolver::get(&input)?;
    let types = types::check(&input, &fqresolvers)?;
    Checker::new(input, fqresolvers, types).check()
}

type TCandidates = HashMap<FQType, ast::NewTypeRef>;

type ACandidates = HashMap<FQSym, ast::GAssignmentRef>;

#[derive(Debug)]
struct ECandidate {
    path: FQPath,
    expr: ExprRef,
}

#[derive(Debug)]
struct Input {
    types: Types,
    symbols: Symbols,
    pkg: Pkg,
    paths: Vec<FQPath>,
    new_types: TCandidates,
    assignments: ACandidates,
    exprs: Vec<ECandidate>,
}

type InputRef = Arc<Input>;

impl Input {
    fn new(env: &Env, ast: &ast::Package) -> Result<InputRef> {
        let mut input = Self {
            types: env.types.clone(),
            symbols: env.symbols.clone(),
            pkg: ast.pkg.clone(),
            paths: Vec::default(),
            new_types: Default::default(),
            assignments: Default::default(),
            exprs: Vec::default(),
        };
        let mut errors = Errors::default();
        for module in &ast.modules {
            for new_type in &module.types {
                let symbol = module.path.fq_type(new_type.it.it.symbol.clone());
                if input.new_types.contains_key(&symbol) {
                    errors.add(new_type.loc.wrap(Error::DuplicateType(symbol)));
                } else {
                    input.new_types.insert(symbol, new_type.clone());
                }
            }
            for assignment in &module.assignments {
                let symbol = module.path.fq_sym(assignment.it.it.symbol.clone());
                if input.assignments.contains_key(&symbol) {
                    errors.add(assignment.loc.wrap(Error::DuplicateSymbol(symbol)));
                } else {
                    input.assignments.insert(symbol, assignment.clone());
                }
            }
            for expr in &module.exprs {
                if module.path.is_empty() {
                    input.exprs.push(ECandidate {
                        path: module.path.clone(),
                        expr: expr.clone(),
                    })
                } else {
                    errors.add(expr.loc.wrap(Error::TLExpressionInNonRootModule));
                    break; // one error per module
                }
            }
            input.paths.push(module.path.clone())
        }
        errors.to_lazy_result(|| Arc::new(input))
    }
}

#[derive(Debug)]
struct Checker {
    input: InputRef,
    fqresolvers: FQResolvers,
    types: Types,
    symbols: Symbols,
    assignments: Vec<L<TLAssignment>>,
    expr: Option<L<Expr>>,
}

impl Checker {
    fn new(input: InputRef, fqresolvers: FQResolvers, types: Types) -> Self {
        let symbols = input.symbols.clone();
        Self {
            input,
            fqresolvers,
            types,
            symbols,
            assignments: Vec::default(),
            expr: None,
        }
    }

    fn new_scope<'a>(&'a self, path: &'a FQPath) -> Scope {
        Scope {
            checker: self,
            path,
            all: Default::default(),
            current: Default::default(),
        }
    }

    fn resolve_type(&self, loc: &Loc, path: &FQPath, symbol: &Q<TSymbol>) -> Result<Type> {
        let fq = self
            .fqresolvers
            .for_path(path)
            .resolve_fq_type(loc, symbol)?;
        if let Some(tipo) = self.types.get(&fq) {
            Ok(tipo.it.clone())
        } else {
            loc.err(Error::UnableToResolveType(symbol.clone()))
        }
    }

    fn resolve_symbol(&self, loc: &Loc, path: &FQPath, symbol: &Q<Symbol>) -> Result<Resolved> {
        let fq = self
            .fqresolvers
            .for_path(path)
            .resolve_fq_symbol(loc, symbol)?;
        if let Some(t) = self.symbols.get(&fq) {
            Ok(Resolved::Global(Global {
                symbol: fq,
                tipo: t.it.clone(),
            }))
        } else {
            loc.err(Error::MissingSymbolDependency(fq))
        }
    }

    fn check(mut self) -> Result<Package> {
        self.check_assignments()?;
        self.check_expressions()?;
        Ok(Package {
            pkg: self.input.pkg.clone(),
            types: self.types,
            symbols: self.symbols,
            assignments: self.assignments,
            expr: self.expr,
        })
    }

    fn check_assignments(&mut self) -> Result<()> {
        let assignments = self.input.assignments.clone();
        loop {
            let mut errors = Errors::default();
            let progress = self.assignments.len();
            for (fq, a) in &assignments {
                if !self.symbols.contains(fq) {
                    errors.add_result(self.check_assignment(fq, a));
                }
            }
            match errors.to_unit_result() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    let missing = e.missing_symbol_deps();
                    if self.assignments.len() == progress || missing.is_empty() {
                        return Err(e);
                    }
                }
            }
        }
    }

    fn check_assignment(&mut self, fq: &FQSym, a: &ast::GAssignmentRef) -> Result<()> {
        let expr = expr::check(&self.new_scope(&fq.path), &a.it.it.expr)?;
        self.symbols
            .set(&a.loc, fq.clone(), a.it.visibility, expr.get_type())?;
        self.assignments.push(a.loc.wrap(TLAssignment {
            symbol: fq.clone(),
            expr,
        }));
        Ok(())
    }

    fn check_expressions(&mut self) -> Result<()> {
        for e in &self.input.exprs {
            if self.expr.is_none() {
                self.expr = Some(expr::check(&self.new_scope(&e.path), &e.expr)?);
            } else {
                return e.expr.loc.err(Error::OnlyOneExpressionAllowed);
            }
        }
        Ok(())
    }
}

enum Resolved {
    Local(Local),
    Global(Global),
}

struct Scope<'a> {
    checker: &'a Checker,
    path: &'a FQPath,
    all: HashMap<Symbol, Type>,
    current: HashMap<Symbol, bool>,
}

impl<'a> Scope<'a> {
    fn resolve_type(&self, loc: &Loc, symbol: &Q<TSymbol>) -> Result<Type> {
        self.checker.resolve_type(loc, self.path, symbol)
    }

    fn resolve_symbol(&self, loc: &Loc, symbol: &Q<Symbol>) -> Result<Resolved> {
        if symbol.segments.is_empty() {
            if let Some(false) = self.current.get(&symbol.symbol) {
                return loc.err(Error::MissingLocalSymbolDependency(symbol.symbol.clone()));
            }
            if let Some(tipo) = self.all.get(&symbol.symbol) {
                return Ok(Resolved::Local(Local {
                    symbol: symbol.symbol.clone(),
                    tipo: tipo.clone(),
                }));
            }
        }
        self.checker.resolve_symbol(loc, self.path, symbol)
    }

    fn child(&self) -> Self {
        Self {
            checker: self.checker,
            path: self.path,
            all: self.all.clone(),
            current: Default::default(),
        }
    }

    fn add_current(&mut self, loc: &Loc, symbol: Symbol) -> Result<()> {
        if self.current.contains_key(&symbol) {
            loc.err(Error::DuplicateLocalSymbol(symbol))
        } else {
            self.current.insert(symbol.clone(), false);
            Ok(())
        }
    }

    fn eval_pending(&self, symbol: &Symbol) -> bool {
        !self.current.get(symbol).cloned().unwrap_or(false)
    }

    fn set(&mut self, loc: &Loc, symbol: Symbol, tipo: Type) -> Result<()> {
        let exists = self.current.get(&symbol);
        match exists {
            None => panic!("Setting symbol [{}] not added as candidate", &symbol),
            Some(true) => loc.err(Error::DuplicateLocalSymbol(symbol)),
            Some(false) => {
                self.current.insert(symbol.clone(), true);
                self.all.insert(symbol, tipo);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests;
