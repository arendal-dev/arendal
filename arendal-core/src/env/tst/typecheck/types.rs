use im::HashMap;

use crate::ast;
use crate::error::{Error, Errors, Result};
use crate::symbol::FQType;
use crate::types::Type;

use crate::visibility::Visibility;

use super::{Input, Module, TypeDefMap, TypeDefinition};

type Candidates<'a> = HashMap<FQType, &'a ast::TypeDefinition>;

pub(super) fn check(input: &Input) -> Result<TypeDefMap> {
    let mut candidates = Candidates::default();
    let mut errors = Errors::default();
    for module in &input.modules {
        for t in &module.ast.types {
            let fq_type = module.path.fq_type(t.symbol.clone());
            if candidates.contains_key(&fq_type) {
                errors.add(t.loc.wrap(Error::DuplicateType(fq_type)));
            } else {
                candidates.insert(fq_type, t);
            }
        }
    }
    errors.to_merged_result(
        Checker {
            input,
            candidates,
            types: TypeDefMap::default(),
            errors: Errors::default(),
        }
        .check(),
    )
}

#[derive(Debug)]
struct Checker<'a> {
    input: &'a Input<'a>,
    candidates: Candidates<'a>,
    types: TypeDefMap,
    errors: Errors,
}

impl<'a> Checker<'a> {
    fn contains_type(&self, fq: &FQType) -> bool {
        self.types.contains_key(fq) || self.input.env.types.contains(fq)
    }

    fn check(mut self) -> Result<TypeDefMap> {
        for (fq, t) in &self.candidates {
            let maybe = if self.contains_type(fq) {
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
