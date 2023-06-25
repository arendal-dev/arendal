use std::collections::HashMap;

use crate::{
    ast::Q,
    error::{Error, Errors, Loc, Result},
    symbol::{FQPath, FQSym, FQType, Pkg, Symbol, TSymbol, FQ},
    visibility::Visibility,
};

use super::Input;

pub(super) fn get<'a, 'b>(input: &'b Input<'a>) -> Result<FQResolvers<'a, 'b>> {
    let mut resolvers = FQResolvers::default();
    for path in &input.paths {
        resolvers.resolvers.insert(
            path.clone(),
            FQResolver {
                path: path.clone(),
                input,
            },
        );
    }
    Ok(resolvers)
}

#[derive(Debug, Default)]
pub(super) struct FQResolvers<'a, 'b> {
    resolvers: HashMap<FQPath, FQResolver<'a, 'b>>,
}

impl<'a, 'b> FQResolvers<'a, 'b> {
    pub(super) fn for_path(&self, path: &FQPath) -> &FQResolver<'a, 'b> {
        self.resolvers.get(path).unwrap()
    }

    pub(super) fn for_symbol<T>(&self, symbol: &FQ<T>) -> &FQResolver<'a, 'b> {
        self.resolvers.get(&symbol.path).unwrap()
    }
}

#[derive(Debug)]
pub(super) struct FQResolver<'a, 'b> {
    path: FQPath,
    input: &'b Input<'a>,
}

impl<'a, 'b> FQResolver<'a, 'b> {
    fn get_candidates<S: Clone, F, B>(&self, b: B, q: &Q<S>) -> Vec<F>
    where
        B: Fn(&FQPath, S) -> F,
    {
        let mut candidates = Vec::default();
        if q.segments.is_empty() {
            candidates.push(b(&self.path, q.symbol.clone()));
            candidates.push(b(&Pkg::Std.empty(), q.symbol.clone()));
        } else {
            todo!();
        }
        candidates
    }

    fn get_type_candidates(&self, q: &Q<TSymbol>) -> Vec<FQType> {
        self.get_candidates(|p, s| p.fq_type(s), q)
    }

    fn get_symbol_candidates(&self, q: &Q<Symbol>) -> Vec<FQSym> {
        self.get_candidates(|p, s| p.fq_sym(s), q)
    }

    fn can_see(&self, visibility: Visibility, path: &FQPath) -> bool {
        self.path.can_see(visibility, path)
    }

    fn get_type_visibility(&self, f: &FQType) -> Option<Visibility> {
        self.input.env.types.get(f).map_or_else(
            || self.input.types.get(f).map(|t| Visibility::Exported), // TODO
            |t| Some(t.visibility),
        )
    }

    pub(super) fn resolve_fq_type(&self, loc: &Loc, q: &Q<TSymbol>) -> Result<FQType> {
        let mut errors = Errors::default();
        for f in self.get_type_candidates(q) {
            if let Some(visibility) = self.get_type_visibility(&f) {
                if self.can_see(visibility, &f.path) {
                    return Ok(f);
                } else {
                    errors.add(loc.wrap(Error::TypeNotVisible(f)))
                }
            }
        }
        errors.to_err(loc.wrap(Error::UnableToResolveType(q.clone())))
    }

    fn get_symbol_visibility(&self, f: &FQSym) -> Option<Visibility> {
        self.input.env.symbols.get(f).map_or_else(
            || self.input.assignments.get(f).map(|a| a.it.visibility),
            |s| Some(s.visibility),
        )
    }

    pub(crate) fn resolve_fq_symbol(&self, loc: &Loc, q: &Q<Symbol>) -> Result<FQSym> {
        let mut errors = Errors::default();
        for f in self.get_symbol_candidates(q) {
            if let Some(visibility) = self.get_symbol_visibility(&f) {
                if self.can_see(visibility, &f.path) {
                    return Ok(f);
                } else {
                    errors.add(loc.wrap(Error::SymbolNotVisible(f)))
                }
            }
        }
        errors.to_err(loc.wrap(Error::UnableToResolveSymbol(q.clone())))
    }
}
