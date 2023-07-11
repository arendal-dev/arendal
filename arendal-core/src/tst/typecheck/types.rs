use crate::{
    ast,
    context::{Context, TypeDfn, TypeDfnMap},
    error::{Errors, Result},
};

use super::{FQResolvers, Input};

pub(super) fn check(input: &Input, fqresolvers: &FQResolvers) -> Result<Context> {
    let errors = Errors::default();
    let mut dfns = TypeDfnMap::default();
    for (symbol, new_type) in &input.new_types {
        let maybe = match new_type.it.it.dfn {
            ast::TypeDfn::Singleton => Some(TypeDfn::Singleton),
        };
        if let Some(checked) = maybe {
            dfns.insert(
                symbol.clone(),
                new_type.loc.wrap(new_type.it.visibility.wrap(checked)),
            );
        }
    }
    input.types.add_types(&errors.to_result(dfns)?)
}
