mod expr;
mod fqresolver;
mod types;

use im::{HashMap, HashSet};

use crate::ast::{self, ExprRef, Q};
use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol};
use crate::types::{Type, Types};

use crate::env::{Env, Symbols};
use crate::visibility::V;

use self::fqresolver::FQResolvers;

use super::{Builder, Expr, Global, Local, Package, TLAssignment, Value};

pub(super) fn check(env: &Env, ast: &ast::Package) -> Result<Package> {
    let input = Input::new(env, ast)?;
    let fqresolvers = fqresolver::get(&input)?;
    let types = types::check(&input, &fqresolvers)?;
    Checker::new(&input, fqresolvers, types).check()
}

type TCandidate<'a> = &'a ast::LVNewType;
type TCandidates<'a> = HashMap<FQType, TCandidate<'a>>;

type ACandidate<'a> = &'a L<V<ast::Assignment>>;
type ACandidates<'a> = HashMap<FQSym, ACandidate<'a>>;

#[derive(Debug)]
struct ECandidate {
    path: FQPath,
    expr: ExprRef,
}

#[derive(Debug)]
struct Input<'a> {
    env: &'a Env,
    pkg: Pkg,
    paths: Vec<FQPath>,
    types: TCandidates<'a>,
    assignments: ACandidates<'a>,
    exprs: Vec<ECandidate>,
}

impl<'a> Input<'a> {
    fn new(env: &'a Env, ast: &'a ast::Package) -> Result<Self> {
        let mut input = Self {
            env,
            pkg: ast.pkg.clone(),
            paths: Vec::default(),
            types: Default::default(),
            assignments: Default::default(),
            exprs: Vec::default(),
        };
        let mut errors = Errors::default();
        for module in &ast.modules {
            for new_type in &module.types {
                let symbol = module.path.fq_type(new_type.it.it.symbol.clone());
                if input.types.contains_key(&symbol) {
                    errors.add(new_type.loc.wrap(Error::DuplicateType(symbol)));
                } else {
                    input.types.insert(symbol, new_type);
                }
            }
            for assignment in &module.assignments {
                let symbol = module.path.fq_sym(assignment.it.it.symbol.clone());
                if input.assignments.contains_key(&symbol) {
                    errors.add(assignment.loc.wrap(Error::DuplicateSymbol(symbol)));
                } else {
                    input.assignments.insert(symbol, assignment);
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
        errors.to_result(input)
    }
}

#[derive(Debug)]
struct Checker<'a, 'b> {
    input: &'b Input<'a>,
    fqresolvers: FQResolvers<'a, 'b>,
    types: Types,
    symbols: Symbols,
    assignments: Vec<L<TLAssignment>>,
    expr: Option<L<Expr>>,
}

impl<'a, 'b> Checker<'a, 'b> {
    fn new(input: &'b Input<'a>, fqresolvers: FQResolvers<'a, 'b>, types: Types) -> Self {
        let symbols = input.env.symbols.clone();
        Self {
            input,
            fqresolvers,
            types,
            symbols,
            assignments: Vec::default(),
            expr: None,
        }
    }

    fn new_scope<'c>(&'c self, path: &'c FQPath) -> Scope<'a, 'b, 'c> {
        Scope {
            checker: self,
            path,
            local: LocalScope::Empty,
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

    fn check_assignment(&mut self, fq: &FQSym, a: ACandidate) -> Result<()> {
        let expr = expr::check(&self.new_scope(&fq.path), a.it.it.expr.clone())?;
        self.symbols
            .set(&a.loc, fq.clone(), a.it.visibility, expr.clone_type())?;
        self.assignments.push(a.loc.wrap(TLAssignment {
            symbol: fq.clone(),
            expr,
        }));
        Ok(())
    }

    fn check_expressions(&mut self) -> Result<()> {
        for e in &self.input.exprs {
            if self.expr.is_none() {
                self.expr = Some(expr::check(&self.new_scope(&e.path), e.expr.clone())?);
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

struct Scope<'a, 'b, 'c> {
    checker: &'c Checker<'a, 'b>,
    path: &'c FQPath,
    local: LocalScope,
}

impl<'a, 'b, 'c> Scope<'a, 'b, 'c> {
    fn resolve_type(&self, loc: &Loc, symbol: &Q<TSymbol>) -> Result<Type> {
        self.checker.resolve_type(loc, self.path, symbol)
    }

    fn resolve_symbol(&self, loc: &Loc, symbol: &Q<Symbol>) -> Result<Resolved> {
        if symbol.segments.is_empty() && self.contains(&symbol.symbol) {
            Ok(Resolved::Local(Local {
                symbol: symbol.symbol.clone(),
                tipo: self.get(&symbol.symbol).unwrap().clone(),
            }))
        } else {
            self.checker.resolve_symbol(loc, self.path, symbol)
        }
    }

    fn child_with(&self, local: LocalScope) -> Self {
        Self {
            checker: self.checker,
            path: self.path,
            local,
        }
    }

    fn child(&self) -> Self {
        self.child_with(match &self.local {
            LocalScope::Empty => LocalScope::First {
                all: Default::default(),
            },
            LocalScope::First { all } | LocalScope::Child { all, .. } => LocalScope::Child {
                all: all.clone(),
                current: Default::default(),
            },
        })
    }

    fn contains(&self, symbol: &Symbol) -> bool {
        match &self.local {
            LocalScope::Empty => false,
            LocalScope::First { all } | LocalScope::Child { all, .. } => all.contains_key(symbol),
        }
    }

    fn get(&self, symbol: &Symbol) -> Option<Type> {
        match &self.local {
            LocalScope::Empty => None,
            LocalScope::First { all } | LocalScope::Child { all, .. } => all.get(symbol),
        }
        .cloned()
    }

    fn set(&mut self, loc: &Loc, symbol: Symbol, tipo: Type) -> Result<()> {
        match &mut self.local {
            LocalScope::Empty => panic!("Can't add local symbols to an empty scope"),
            LocalScope::First { all } => {
                if all.contains_key(&symbol) {
                    loc.err(Error::DuplicateLocalSymbol(symbol))
                } else {
                    all.insert(symbol, tipo);
                    Ok(())
                }
            }
            LocalScope::Child { all, current } => {
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

#[derive(Debug, Clone)]
enum LocalScope {
    Empty,
    First {
        all: HashMap<Symbol, Type>,
    },
    Child {
        all: HashMap<Symbol, Type>,
        current: HashSet<Symbol>,
    },
}

#[cfg(test)]
mod tests;
