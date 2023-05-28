use crate::{symbol::FQType, visibility::Visibility};

use super::{Type, TypeMap};

pub(super) fn load_types() -> TypeMap {
    TypeLoader::default().load()
}

#[derive(Default)]
struct TypeLoader {
    types: TypeMap,
}

impl TypeLoader {
    fn load(mut self) -> TypeMap {
        self.export(FQType::None, Type::None);
        self.export(FQType::True, Type::True);
        self.export(FQType::False, Type::False);
        self.export(FQType::Boolean, Type::Boolean);
        self.export(FQType::Integer, Type::Integer);
        self.types
    }

    fn export(&mut self, symbol: FQType, tipo: Type) {
        self.types.insert(symbol, Visibility::Exported.wrap(tipo));
    }
}
