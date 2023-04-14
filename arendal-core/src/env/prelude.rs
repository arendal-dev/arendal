use super::{Types, Visibility};
use crate::{
    error::{Loc, Result},
    types::Type,
};

fn add_prelude_type(types: &mut Types, tipo: Type) -> Result<()> {
    types.add(Loc::none(), Visibility::Exported, tipo)
}

pub(super) fn add_prelude_types(types: &mut Types) -> Result<()> {
    add_prelude_type(types, Type::None)?;
    add_prelude_type(types, Type::True)?;
    add_prelude_type(types, Type::False)?;
    add_prelude_type(types, Type::Boolean)?;
    add_prelude_type(types, Type::Integer)
}
