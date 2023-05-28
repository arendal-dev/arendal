use im::HashMap;

use crate::ast;
use crate::error::{Error, Errors, Result};
use crate::symbol::FQType;
use crate::types::{TypeDfn, TypeDfnMap, Types};

use crate::visibility::Visibility;

use super::Input;

type Candidates<'a> = HashMap<FQType, &'a ast::TypeDefinition>;

pub(super) fn check(input: &Input) -> Result<Types> {
    let mut candidates = Candidates::default();
    let mut errors = Errors::default();
    for module in &input.modules {
        for t in &module.ast.types {
            let fq_type = module.path.fq_type(t.symbol.clone());
            if candidates.contains_key(&fq_type) {
                errors.add(t.loc.wrap(Error::DuplicateLocalType(t.symbol.clone())));
            } else {
                candidates.insert(fq_type, t);
            }
        }
    }
    let mut dfns = TypeDfnMap::default();
    for (fq, t) in candidates {
        let maybe = match t.dfn {
            ast::TypeDfn::Singleton => Some(TypeDfn::Singleton),
        };
        if let Some(dfn) = maybe {
            dfns.insert(fq.clone(), t.loc.wrap(Visibility::Module.wrap(dfn)));
        }
    }
    input.env.types.add_types(&errors.to_result(dfns)?)
}
