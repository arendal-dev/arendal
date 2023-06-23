use crate::{
    ast::{self, Q},
    error::Result,
    symbol::{FQPath, Pkg},
};

pub(super) fn get_resolvers(input: &ast::Package) -> Result<Resolvers> {
    let mut resolvers = Vec::default();
    for (local, module) in &input.modules {
        let path = input.pkg.path(local.clone());
        resolvers.push(Resolver { path, module });
    }
    Ok(Resolvers {
        pkg: input.pkg.clone(),
        resolvers,
    })
}

#[derive(Debug)]
pub(super) struct Resolvers<'a> {
    pub(super) pkg: Pkg,
    pub(super) resolvers: Vec<Resolver<'a>>,
}
#[derive(Debug)]
pub(super) struct Resolver<'a> {
    pub(super) path: FQPath,
    pub(super) module: &'a ast::Module,
}

impl<'a> Resolver<'a> {
    fn get_all_candidates<S: Clone, F, A, B>(&self, mut accumulator: A, b: B, q: &Q<S>)
    where
        A: FnMut(F),
        B: Fn(&FQPath, S) -> F,
    {
        if q.segments.is_empty() {
            accumulator(b(&self.path, q.symbol.clone()));
            accumulator(b(&Pkg::Std.empty(), q.symbol.clone()));
        } else {
            todo!();
        }
    }

    pub(super) fn get_candidates<S: Clone, F, B, C>(&self, check: C, b: B, q: &Q<S>) -> Vec<F>
    where
        C: Fn(&F) -> bool,
        B: Fn(&FQPath, S) -> F,
    {
        let mut result = Vec::default();
        self.get_all_candidates(
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
}
