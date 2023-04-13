use super::{Module, PkgRef, TTarget, Visibility};
use crate::{
    error::{Loc, Result},
    symbol::{ModulePath, TSymbol},
    symbols::TSymbols,
    types::Type,
};

fn add_tl_tsymbol(module: &mut Module, s: TSymbols, tipo: Type) -> Result<()> {
    let symbol = TSymbol::known(s);
    module.add_tsymbol(
        Loc::none(),
        symbol,
        TTarget::tipo(Visibility::Exported, tipo),
    )
}

pub(super) fn load_prelude(pkg: &PkgRef) -> Result<()> {
    let mut std = pkg.create_module(Loc::none(), ModulePath::empty())?;
    add_tl_tsymbol(&mut std, TSymbols::None, Type::None)?;
    add_tl_tsymbol(&mut std, TSymbols::True, Type::True)?;
    add_tl_tsymbol(&mut std, TSymbols::False, Type::False)?;
    add_tl_tsymbol(&mut std, TSymbols::Boolean, Type::Boolean)?;
    add_tl_tsymbol(&mut std, TSymbols::Integer, Type::Integer)
}
