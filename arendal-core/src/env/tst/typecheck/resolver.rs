use crate::{
    ast::{self, Package, Q},
    error::{Error, Errors, Loc, Result},
    symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol},
};

#[derive(Debug)]
pub(super) struct Module<'a> {
    pub(super) path: FQPath,
    pub(super) ast: &'a ast::Module,
}

pub(super) fn get_modules(input: &Package) -> Result<Vec<Module>> {
    let mut errors = Errors::default();
    let mut modules = Vec::default();
    for (path, module) in &input.modules {
        if let Some(m) = errors.add_result(Module::new(input.pkg.path(path.clone()), module)) {
            modules.push(m);
        }
    }
    errors.to_result(modules)
}

impl<'a> Module<'a> {
    fn new(path: FQPath, ast: &ast::Module) -> Result<Module> {
        Ok(Module { path, ast })
    }

    fn get_candidates<S: Clone, F, B>(&self, b: B, q: &Q<S>) -> Vec<F>
    where
        B: Fn(&FQPath, S) -> F,
    {
        let mut result = Vec::default();
        if q.segments.is_empty() {
            result.push(b(&self.path, q.symbol.clone()));
            result.push(b(&Pkg::Std.empty(), q.symbol.clone()));
        } else {
            todo!();
        }
        result
    }

    fn resolve<S: Clone, F, B, C>(&self, check: C, b: B, q: &Q<S>) -> Option<F>
    where
        C: Fn(&F) -> bool,
        B: Fn(&FQPath, S) -> F,
    {
        for candidate in self.get_candidates(b, q) {
            if check(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    pub(super) fn resolve_type<F>(&self, types: F, loc: &Loc, q: &Q<TSymbol>) -> Result<FQType>
    where
        F: Fn(&FQType) -> bool,
    {
        match self.resolve(types, |p, s| p.fq_type(s), q) {
            Some(fq) => Ok(fq),
            None => loc.err(Error::UnableToResolveType(q.clone())),
        }
    }

    pub(super) fn resolve_symbol<F>(&self, symbols: F, loc: &Loc, q: &Q<Symbol>) -> Result<FQSym>
    where
        F: Fn(&FQSym) -> bool,
    {
        match self.resolve(symbols, |p, s| p.fq_sym(s), q) {
            Some(fq) => Ok(fq),
            None => loc.err(Error::UnableToResolveSymbol(q.clone())),
        }
    }
}
