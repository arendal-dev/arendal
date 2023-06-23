use crate::{types::Type, visibility::Visibility};

use super::TypeMap;

pub(super) fn get_builtin_types() -> TypeMap {
    TypeLoader::default().load()
}

#[derive(Default)]
struct TypeLoader {
    types: TypeMap,
}

impl TypeLoader {
    fn load(mut self) -> TypeMap {
        self.export(Type::None);
        self.export(Type::True);
        self.export(Type::False);
        self.export(Type::Boolean);
        self.export(Type::Integer);
        self.types
    }

    fn builtin(&mut self, tipo: Type) {}

    fn export(&mut self, tipo: Type) {
        self.types
            .insert(tipo.fq(), Visibility::Exported.wrap(tipo));
    }
}
