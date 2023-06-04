mod expr;
mod module;
mod types;

use std::collections::{HashMap, HashSet};

use crate::ast::{self, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol};
use crate::types::{Type, Types};

use crate::env::{Env, Symbols};
use crate::visibility::V;

use self::module::Module;
use super::{BStmt, Expr, ExprBuilder, Package, TLAssignment, Value};

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

struct ACandidate<'a> {
    assignment: &'a L<V<ast::Assignment>>,
    deps: HashSet<FQSym>,
}

impl<'a> ACandidate<'a> {
    fn new(assignment: &'a L<V<ast::Assignment>>) -> Self {
        Self {
            assignment,
            deps: Default::default(),
        }
    }
}

type ACandidates<'a> = HashMap<FQSym, ACandidate<'a>>;

struct ECandidate<'a> {
    path: FQPath,
    expr: &'a L<Expr>,
}

type ECandidates<'a> = Vec<ACandidate<'a>>;

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
        let mut assignments = Vec::default();
        let mut exprs = Vec::default();
        let mut errors = Errors::default();
        for module in &self.input.modules {
            let mut checker = ModuleChecker::new(&self, module);
            errors.add_result(checker.check());
            assignments.append(&mut checker.assignments);
            exprs.append(&mut checker.exprs);
        }
        errors.to_lazy_result(|| Package {
            pkg: self.input.pkg.clone(),
            types: self.types,
            assignments,
            exprs,
        })
    }
}

struct PackageChecker2<'a> {
    pkg: Pkg,
    types: Types,
    symbols: Symbols,
    t_candidates: TCandidates<'a>,
    a_candidates: ACandidates<'a>,
    e_candidates: Vec<&'a L<ast::Expr>>,
}

impl<'a> PackageChecker2<'a> {
    fn new(env: &Env, input: &'a ast::Package) -> Result<Self> {
        let mut errors = Errors::default();
        let mut t_candidates = TCandidates::default();
        let mut a_candidates = ACandidates::default();
        let mut e_candidates = Vec::default();
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
            for a in &module.assignments {
                let fq = fq_path.fq_sym(a.it.it.symbol.clone());
                if a_candidates.contains_key(&fq) {
                    errors.add(a.loc.wrap(Error::DuplicateSymbol(fq)));
                } else {
                    a_candidates.insert(fq, ACandidate::new(a));
                }
            }
            for e in &module.exprs {
                if path.is_empty() {
                    e_candidates.push(e)
                } else {
                    errors.add(e.loc.wrap(Error::TLExpressionInNonRootModule));
                    break; // one error per module
                }
            }
        }
        errors.to_lazy_result(|| Self {
            pkg: input.pkg.clone(),
            types: env.types.clone(),
            symbols: env.symbols.clone(),
            t_candidates,
            a_candidates,
            e_candidates,
        })
    }

    fn get_all_candidates<S: Clone, F, A, B>(
        &self,
        path: &FQPath,
        mut accumulator: A,
        b: B,
        q: &Q<S>,
    ) where
        A: FnMut(F),
        B: Fn(&FQPath, S) -> F,
    {
        if q.segments.is_empty() {
            accumulator(b(&path, q.symbol.clone()));
            accumulator(b(&Pkg::Std.empty(), q.symbol.clone()));
        } else {
            todo!();
        }
    }

    fn get_candidates<S: Clone, F, B, C>(&self, path: &FQPath, check: C, b: B, q: &Q<S>) -> Vec<F>
    where
        C: Fn(&F) -> bool,
        B: Fn(&FQPath, S) -> F,
    {
        let mut result = Vec::default();
        self.get_all_candidates(
            path,
            |f| {
                if check(&f) {
                    result.push(f)
                }
            },
            b,
            q,
        );
        result
    }

    fn get_type_candidates(&self, path: &FQPath, q: &Q<TSymbol>) -> Vec<FQType> {
        self.get_candidates(
            path,
            |f| self.t_candidates.contains_key(f) || self.types.contains(f),
            |p, s| p.fq_type(s),
            q,
        )
    }

    fn get_symbol_candidates(&self, path: &FQPath, q: &Q<Symbol>) -> Vec<FQSym> {
        self.get_candidates(
            path,
            |f| self.a_candidates.contains_key(f) || self.symbols.contains(f),
            |p, s| p.fq_sym(s),
            q,
        )
    }

    fn resolve_type(&self, loc: &Loc, path: &FQPath, q: &Q<TSymbol>) -> Result<FQType> {
        for f in self.get_type_candidates(path, q) {
            return Ok(f); // TODO: validate visibility
        }
        loc.err(Error::UnableToResolveType(q.clone()))
    }

    fn resolve_symbol(&self, loc: &Loc, path: &FQPath, q: &Q<Symbol>) -> Result<FQSym> {
        for f in self.get_symbol_candidates(path, q) {
            return Ok(f); // TODO: validate visibility
        }
        loc.err(Error::UnableToResolveSymbol(q.clone()))
    }
}

type Scope = crate::env::Scope<Type>;

#[derive(Debug)]
struct ModuleChecker<'a> {
    pkg: &'a PackageChecker<'a>,
    input: &'a Module<'a>,
    scope: Scope,
    assignments: Vec<L<TLAssignment>>,
    exprs: Vec<L<Expr>>,
}

impl<'a> ModuleChecker<'a> {
    fn new(pkg: &'a PackageChecker, input: &'a Module) -> ModuleChecker<'a> {
        ModuleChecker {
            pkg,
            input,
            scope: Scope::default(),
            assignments: Vec::default(),
            exprs: Vec::default(),
        }
    }

    fn check(&mut self) -> Result<()> {
        for a in &self.input.ast.assignments {
            let checked = self.check_assignment(a)?;
            self.assignments.push(checked)
        }
        for e in &self.input.ast.exprs {
            let checked = self.check_expression(e)?;
            self.exprs.push(checked)
        }
        Ok(())
    }

    fn check_assignment(&mut self, a: &L<V<ast::Assignment>>) -> Result<L<TLAssignment>> {
        let symbol = a.it.it.symbol.clone();
        if self.scope.contains(&symbol) {
            a.loc
                .err(Error::DuplicateSymbol(self.input.path.fq_sym(symbol)))
        } else {
            let expr = expr::check(self, &self.scope, &a.it.it.expr)?;
            self.scope.set(&a.loc, symbol.clone(), expr.clone_type())?;
            Ok(a.loc.wrap(TLAssignment {
                symbol: self.input.path.fq_sym(symbol),
                expr,
            }))
        }
    }

    fn check_expression(&mut self, e: &L<ast::Expr>) -> Result<L<Expr>> {
        Ok(expr::check(self, &self.scope, e)?)
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
