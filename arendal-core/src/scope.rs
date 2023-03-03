use std::collections::HashMap;

use crate::types;
use crate::types::Type;
use crate::ArcStr;

struct Scope {
    types: HashMap<ArcStr, Type>,
}

impl Scope {
    fn add_type(&mut self, name: &ArcStr, tipo: Type) {
        self.types.insert(name.clone(), tipo);
    }

    fn add_builtin_types(&mut self) {
        self.add_type(&types::BOOLEAN, Type::boolean());
        self.add_type(&types::TRUE, Type::boolean_true());
        self.add_type(&types::FALSE, Type::boolean_false());
        self.add_type(&types::INTEGER, Type::integer());
        self.add_type(&types::NONE, Type::none());
    }
}
