use arcstr::ArcStr;

use super::{Module, PkgRef, TTarget, Visibility};
use crate::{
    error::{Loc, Result},
    symbol::{self, ModulePath, TSymbol},
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
    add_tl_tsymbol(&mut std, &symbol::BOOLEAN, Type::Boolean)?;
    add_tl_tsymbol(&mut std, &symbol::TRUE, Type::True)?;
    add_tl_tsymbol(&mut std, &symbol::FALSE, Type::False)?;
    add_tl_tsymbol(&mut std, &symbol::INTEGER, Type::Integer)?;
    add_tl_tsymbol(&mut std, &symbol::NONE, Type::None)
}
