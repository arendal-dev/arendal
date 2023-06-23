mod expr;
mod resolver;

use im::{HashMap, HashSet};

use crate::ast::{self, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, FQSym, FQType, Symbol, TSymbol};
use crate::types::{Type, TypeDfn, TypeDfnMap, Types};

use crate::env::{Env, Symbols};
use crate::visibility::{Visibility, V};

use self::resolver::{Resolver, Resolvers};

use super::{Builder, Expr, Global, Package, TLAssignment, Value};

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<Package> {
    let resolvers = resolver::get_resolvers(input)?;
    TypeChecker::new(env, resolvers)?.check()
}

#[derive(Debug)]
struct TCandidate<'a> {
    dfn: &'a ast::TypeDefinition,
}

type TCandidates<'a> = std::collections::HashMap<FQType, TCandidate<'a>>;

type ACandidate<'a> = &'a L<V<ast::Assignment>>;
type ACandidates<'a> = HashMap<FQSym, ACandidate<'a>>;

#[derive(Debug)]
struct ECandidate<'a> {
    path: FQPath,
    expr: &'a L<Expr>,
}

type ECandidates<'a> = Vec<ACandidate<'a>>;

#[derive(Debug)]
struct TypeChecker<'a> {
    input: Resolvers<'a>,
    types: Types,
    symbols: Symbols,
    assignments: Vec<L<TLAssignment>>,
    expr: Option<L<Expr>>,
    t_candidates: TCandidates<'a>,
    a_candidates: ACandidates<'a>,
    e_candidates: Vec<&'a L<ast::Expr>>,
}

impl<'a> TypeChecker<'a> {
    fn new(env: &Env, input: Resolvers<'a>) -> Result<Self> {
        let mut errors = Errors::default();
        let mut t_candidates = TCandidates::default();
        let mut a_candidates = ACandidates::default();
        let mut e_candidates = Vec::default();
        for resolver in &input.resolvers {
            for dfn in &resolver.module.types {
                let fq_type = resolver.path.fq_type(dfn.symbol.clone());
                if t_candidates.contains_key(&fq_type) {
                    errors.add(dfn.loc.wrap(Error::DuplicateType(fq_type)));
                } else {
                    t_candidates.insert(fq_type, TCandidate { dfn });
                }
            }
            for a in &resolver.module.assignments {
                let fq = resolver.path.fq_sym(a.it.it.symbol.clone());
                if a_candidates.contains_key(&fq) {
                    errors.add(a.loc.wrap(Error::DuplicateSymbol(fq)));
                } else {
                    a_candidates.insert(fq, a);
                }
            }
            for e in &resolver.module.exprs {
                if resolver.path.is_empty() {
                    e_candidates.push(e)
                } else {
                    errors.add(e.loc.wrap(Error::TLExpressionInNonRootModule));
                    break; // one error per module
                }
            }
        }
        errors.to_lazy_result(|| Self {
            input,
            types: env.types.clone(),
            symbols: env.symbols.clone(),
            assignments: Vec::default(),
            expr: None,
            t_candidates,
            a_candidates,
            e_candidates,
        })
    }

    fn get_type_candidates(&self, resolver: &Resolver, q: &Q<TSymbol>) -> Vec<FQType> {
        resolver.get_candidates(
            |f| self.t_candidates.contains_key(f) || self.types.contains(f),
            |p, s| p.fq_type(s),
            q,
        )
    }

    fn get_symbol_candidates(&self, resolver: &Resolver, q: &Q<Symbol>) -> Vec<FQSym> {
        resolver.get_candidates(
            |f| self.a_candidates.contains_key(f) || self.symbols.contains(f),
            |p, s| p.fq_sym(s),
            q,
        )
    }

    fn resolve_fq_type(&self, loc: &Loc, resolver: &Resolver, q: &Q<TSymbol>) -> Result<FQType> {
        for f in self.get_type_candidates(resolver, q) {
            return Ok(f); // TODO: validate visibility
        }
        loc.err(Error::UnableToResolveType(q.clone()))
    }

    fn resolve_type(&self, loc: &Loc, resolver: &Resolver, q: &Q<TSymbol>) -> Result<Type> {
        let fq = self.resolve_fq_type(loc, resolver, q)?;
        if let Some(t) = self.types.get(&fq) {
            Ok(t.it.clone())
        } else {
            loc.err(Error::UnableToResolveType(q.clone()))
        }
    }

    fn resolve_fq_symbol(&self, loc: &Loc, resolver: &Resolver, q: &Q<Symbol>) -> Result<FQSym> {
        let mut errors = Errors::default();
        for f in self.get_symbol_candidates(resolver, q) {
            if let Some(s) = self.symbols.get(&f) {
                if resolver.path.can_see(s.visibility, &f.path) {
                    return Ok(f);
                } else {
                    errors.add(loc.wrap(Error::SymbolNotVisible(f)))
                }
            } else {
                return loc.err(Error::MissingSymbolDependency(f));
            }
        }
        errors.add(loc.wrap(Error::UnableToResolveSymbol(q.clone())));
        errors.to_err()
    }

    fn resolve_global(&self, loc: &Loc, resolver: &Resolver, q: &Q<Symbol>) -> Result<Global> {
        let fq = self.resolve_fq_symbol(loc, resolver, q)?;
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
            pkg: self.input.pkg,
            types: self.types,
            symbols: self.symbols,
            assignments: self.assignments,
            expr: self.expr,
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
        let candidates = self.a_candidates.clone();
        loop {
            let mut errors = Errors::default();
            let progress = self.assignments.len();
            for (fq, a) in &candidates {
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

    fn check_assignment(&mut self, fq: &FQSym, a: ACandidate) -> Result<()> {
        // TODO: use right resolver!!
        let expr = expr::check(self, &self.input.resolvers[0], &Scope::Empty, &a.it.it.expr)?;
        self.symbols
            .set(&a.loc, fq.clone(), a.it.visibility, expr.clone_type())?;
        self.assignments.push(a.loc.wrap(TLAssignment {
            symbol: fq.clone(),
            expr,
        }));
        Ok(())
    }

    fn check_expressions(&mut self) -> Result<()> {
        let path = self.input.pkg.empty();
        for e in &self.e_candidates {
            if self.expr.is_none() {
                // TODO: use right resolver!!
                self.expr = Some(expr::check(
                    self,
                    &self.input.resolvers[0],
                    &Scope::Empty,
                    e,
                )?);
            } else {
                return e.loc.err(Error::OnlyOneExpressionAllowed);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum Scope {
    Empty,
    First {
        all: HashMap<Symbol, Type>,
    },
    Child {
        all: HashMap<Symbol, Type>,
        current: HashSet<Symbol>,
    },
}

impl Scope {
    fn child(&self) -> Self {
        match self {
            Self::Empty => Self::First {
                all: Default::default(),
            },
            Self::First { all } | Self::Child { all, .. } => Self::Child {
                all: all.clone(),
                current: Default::default(),
            },
        }
    }

    fn contains(&self, symbol: &Symbol) -> bool {
        match self {
            Self::Empty => false,
            Self::First { all } | Self::Child { all, .. } => all.contains_key(symbol),
        }
    }

    fn get(&self, symbol: &Symbol) -> Option<Type> {
        match self {
            Self::Empty => None,
            Self::First { all } | Self::Child { all, .. } => all.get(symbol),
        }
        .cloned()
    }

    fn set(&mut self, loc: &Loc, symbol: Symbol, tipo: Type) -> Result<()> {
        match self {
            Self::Empty => panic!("Can't add local symbols to an empty scope"),
            Self::First { all } => {
                if all.contains_key(&symbol) {
                    loc.err(Error::DuplicateLocalSymbol(symbol))
                } else {
                    all.insert(symbol, tipo);
                    Ok(())
                }
            }
            Self::Child { all, current } => {
                if current.contains(&symbol) {
                    loc.err(Error::DuplicateLocalSymbol(symbol))
                } else {
                    current.insert(symbol.clone());
                    all.insert(symbol, tipo);
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
