use std::collections::HashMap;

use crate::id::{FQTypeId, TypeId};
use crate::types::Type;

#[derive(Debug, Clone)]
enum Kind {
    Type(Type),
}

#[derive(Clone, Default)]
pub struct Scope {
    fq_kinds: HashMap<FQTypeId, Kind>,
}

impl Scope {
    pub fn builtin() -> Self {
        let mut scope: Scope = Default::default();
        scope.add_builtin_types();
        scope
    }

    fn add_fq_kind(&mut self, id: &FQTypeId, kind: Kind) -> Result<(), ScopeError> {
        let id = id.clone();
        if self.fq_kinds.contains_key(&id) {
            Err(ScopeError::DuplicateFQTypeId(id))
        } else {
            self.fq_kinds.insert(id, kind);
            Ok(())
        }
    }

    fn add_type(&mut self, name: &TypeId, tipo: Type) -> Result<(), ScopeError> {
        todo!()
    }

    fn add_builtin_types(&mut self) {
        /*
        self.add_type(&BOOLEAN, Type::boolean()).unwrap();
        self.add_type(&TRUE, Type::boolean_true()).unwrap();
        self.add_type(&FALSE, Type::boolean_false()).unwrap();
        self.add_type(&INTEGER, Type::integer()).unwrap();
        self.add_type(&NONE, Type::none()).unwrap();
        */
        todo!()
    }
}

#[derive(Debug)]
pub enum ScopeError {
    DuplicateFQTypeId(FQTypeId),
}

impl crate::error::Error for ScopeError {}
