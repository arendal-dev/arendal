use crate::{
    ast,
    error::{Errors, Result},
    types::{TypeDfn, TypeDfnMap, Types},
    visibility::Visibility,
};

use super::{FQResolvers, Input};

pub(super) fn check<'a, 'b>(input: &Input<'a>, fqresolvers: &FQResolvers<'a, 'b>) -> Result<Types> {
    let errors = Errors::default();
    let mut dfns = TypeDfnMap::default();
    for (symbol, dfn) in &input.types {
        let maybe = match dfn.dfn {
            ast::TypeDfn::Singleton => Some(TypeDfn::Singleton),
        };
        if let Some(checked) = maybe {
            dfns.insert(
                symbol.clone(),
                dfn.loc.wrap(Visibility::Module.wrap(checked)),
            );
        }
    }
    input.env.types.add_types(&errors.to_result(dfns)?)
}
