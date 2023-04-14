use super::{Module, PkgRef, Target, Visibility};
use crate::{
    error::{Loc, Result},
    symbol::{ModulePath, TSymbol},
    types::Type,
};

fn add_tl_tsymbol(module: &mut Module, symbol: TSymbol, tipo: Type) -> Result<()> {
    module.add_tsymbol(Loc::none(), symbol, Target::new(Visibility::Exported, tipo))
}

pub(super) fn load_prelude(pkg: &PkgRef) -> Result<()> {
    let mut std = pkg.create_module(Loc::none(), ModulePath::empty())?;
    add_tl_tsymbol(&mut std, TSymbol::None, Type::None)?;
    add_tl_tsymbol(&mut std, TSymbol::True, Type::True)?;
    add_tl_tsymbol(&mut std, TSymbol::False, Type::False)?;
    add_tl_tsymbol(&mut std, TSymbol::Boolean, Type::Boolean)?;
    add_tl_tsymbol(&mut std, TSymbol::Integer, Type::Integer)
}
