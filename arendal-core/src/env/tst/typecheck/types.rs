use crate::ast;
use crate::error::{Error, Errors, Result};
use crate::symbol::FQType;
use crate::types::Type;

use crate::env::Env;
use crate::visibility::Visibility;

use super::{TypeDefMap, TypeDefinition};

type Candidate<'a> = (FQType, &'a ast::TypeDefinition);
type Candidates<'a> = Vec<Candidate<'a>>;

pub(super) fn check(env: &Env, input: &ast::Package) -> Result<TypeDefMap> {
    let mut candidates = Candidates::default();
    for (path, module) in &input.modules {
        let fqpath = input.pkg.path(path.clone());
        for t in &module.types {
            candidates.push((fqpath.fq_type(t.symbol.clone()), t));
        }
    }
    Checker {
        env,
        candidates,
        types: TypeDefMap::default(),
        errors: Errors::default(),
    }
    .check()
}

#[derive(Debug)]
struct Checker<'a> {
    env: &'a Env,
    candidates: Candidates<'a>,
    types: TypeDefMap,
    errors: Errors,
}

impl<'a> Checker<'a> {
    fn check(mut self) -> Result<TypeDefMap> {
        for (fq, t) in &self.candidates {
            let maybe = if self.types.contains_key(fq) || self.env.types.contains(fq) {
                self.errors
                    .add(t.loc.wrap(Error::DuplicateType(fq.clone())));
                None
            } else {
                match t.dfn {
                    ast::TypeDfn::Singleton => {
                        self.errors.add_result(Type::singleton(&t.loc, fq.clone()))
                    }
                }
            };
            if let Some(tipo) = maybe {
                self.types.insert(
                    fq.clone(),
                    TypeDefinition {
                        loc: t.loc.clone(),
                        visibility: Visibility::Module,
                        tipo,
                    },
                );
            }
        }
        self.errors.to_result(self.types)
    }
}
