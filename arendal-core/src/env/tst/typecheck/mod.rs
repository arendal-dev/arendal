mod expr;

use std::collections::{HashMap, HashSet};

use crate::ast::{self, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol};
use crate::types::{Type, TypeDfn, TypeDfnMap, Types};

use crate::env::{Env, Symbols};
use crate::visibility::{Visibility, V};

use super::{BStmt, Builder, Expr, Global, Package, TLAssignment, Value};

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<Package> {
    TypeChecker::new(env, input)?.check()
}

#[derive(Debug)]
struct TCandidate<'a> {
    dfn: &'a ast::TypeDefinition,
}

type TCandidates<'a> = HashMap<FQType, TCandidate<'a>>;

#[derive(Debug)]
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

#[derive(Debug)]
struct ECandidate<'a> {
    path: FQPath,
    expr: &'a L<Expr>,
}

type ECandidates<'a> = Vec<ACandidate<'a>>;

type Scope = crate::env::Scope<Type>;

#[derive(Debug)]
struct TypeChecker<'a> {
    pkg: Pkg,
    types: Types,
    symbols: Symbols,
    assignments: Vec<L<TLAssignment>>,
    exprs: Vec<L<Expr>>,
    t_candidates: TCandidates<'a>,
    a_candidates: ACandidates<'a>,
    e_candidates: Vec<&'a L<ast::Expr>>,
    scope: Scope,
}

impl<'a> TypeChecker<'a> {
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
            assignments: Vec::default(),
            exprs: Vec::default(),
            t_candidates,
            a_candidates,
            e_candidates,
            scope: Scope::default(),
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

    fn resolve_fq_type(&self, loc: &Loc, path: &FQPath, q: &Q<TSymbol>) -> Result<FQType> {
        for f in self.get_type_candidates(path, q) {
            return Ok(f); // TODO: validate visibility
        }
        loc.err(Error::UnableToResolveType(q.clone()))
    }

    fn resolve_type(&self, loc: &Loc, path: &FQPath, q: &Q<TSymbol>) -> Result<Type> {
        let fq = self.resolve_fq_type(loc, path, q)?;
        if let Some(t) = self.types.get(&fq) {
            Ok(t.it.clone())
        } else {
            loc.err(Error::UnableToResolveType(q.clone()))
        }
    }

    fn resolve_fq_symbol(&self, loc: &Loc, path: &FQPath, q: &Q<Symbol>) -> Result<FQSym> {
        for f in self.get_symbol_candidates(path, q) {
            return Ok(f); // TODO: validate visibility
        }
        loc.err(Error::UnableToResolveSymbol(q.clone()))
    }

    fn resolve_global(&self, loc: &Loc, path: &FQPath, q: &Q<Symbol>) -> Result<Global> {
        let fq = self.resolve_fq_symbol(loc, path, q)?;
        if let Some(t) = self.symbols.get(&fq) {
            Ok(Global {
                symbol: fq,
                tipo: t.it.clone(),
            })
        } else {
            loc.err(Error::UnableToResolveSymbol(q.clone()))
        }
    }

    fn check(mut self) -> Result<Package> {
        self.types = self.check_types()?;
        self.check_assignments()?;
        self.check_expressions()?;
        Ok(Package {
            pkg: self.pkg,
            types: self.types,
            symbols: self.symbols,
            assignments: self.assignments,
            exprs: self.exprs,
        })
    }

    fn check_types(&self) -> Result<Types> {
        let errors = Errors::default();
        let mut dfns = TypeDfnMap::default();
        for (fq, t) in &self.t_candidates {
            let maybe = match t.dfn.dfn {
                ast::TypeDfn::Singleton => Some(TypeDfn::Singleton),
            };
            if let Some(dfn) = maybe {
                dfns.insert(fq.clone(), t.dfn.loc.wrap(Visibility::Module.wrap(dfn)));
            }
        }
        self.types.add_types(&errors.to_result(dfns)?)
    }

    fn check_assignments(&mut self) -> Result<()> {
        let mut errors = Errors::default();
        for (fq, c) in &self.a_candidates {
            let a = c.assignment;
            let path = fq.path();
            if let Some(expr) =
                errors.add_result(expr::check(self, &path, &self.scope, &a.it.it.expr))
            {
                if errors
                    .add_result(self.symbols.set(
                        &a.loc,
                        fq.clone(),
                        a.it.visibility,
                        expr.clone_type(),
                    ))
                    .is_some()
                {
                    self.assignments.push(a.loc.wrap(TLAssignment {
                        symbol: fq.clone(),
                        expr,
                    }));
                }
            }
        }
        errors.to_unit_result()
    }

    fn check_expressions(&mut self) -> Result<()> {
        let path = self.pkg.empty();
        let mut errors = Errors::default();
        for e in &self.e_candidates {
            if let Some(expr) = errors.add_result(expr::check(self, &path, &self.scope, e)) {
                self.exprs.push(expr)
            }
        }
        errors.to_unit_result()
    }
}

#[cfg(test)]
mod tests;
