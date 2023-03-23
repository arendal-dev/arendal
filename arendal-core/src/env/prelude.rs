use arcstr::ArcStr;

use super::{Module, PkgRef, TTarget, Visibility};
use crate::{
    error::{Loc, Result},
    symbol::{ModulePath, TSymbol},
    types::{self, Type},
};

fn add_tl_tsymbol(module: &mut Module, name: &ArcStr, tipo: Type) -> Result<()> {
    let symbol = TSymbol::new(Loc::none(), name.clone())?;
    module.add_tsymbol(
        Loc::none(),
        symbol,
        TTarget::tipo(Visibility::Exported, tipo),
    )
}

pub(super) fn load_prelude(pkg: &PkgRef) -> Result<()> {
    let mut std = pkg.create_module(Loc::none(), ModulePath::empty())?;
    add_tl_tsymbol(&mut std, &types::BOOLEAN, Type::boolean())?;
    add_tl_tsymbol(&mut std, &types::TRUE, Type::boolean_true())?;
    add_tl_tsymbol(&mut std, &types::FALSE, Type::boolean_false())?;
    add_tl_tsymbol(&mut std, &types::INTEGER, Type::integer())?;
    add_tl_tsymbol(&mut std, &types::NONE, Type::none())
}
