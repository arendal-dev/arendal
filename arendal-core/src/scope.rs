use std::collections::HashMap;
use std::rc::Rc;

use crate::id::{FQTypeId, TypeId};
use crate::names::*;
use crate::types::Type;
use crate::ArcStr;

#[derive(Debug, Clone)]
pub struct Closed {
    scope: Rc<Scope>,
}

#[derive(Debug, Clone)]
enum Kind {
    Type(Type),
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    fq_kinds: HashMap<FQTypeId, Kind>,
    local_kinds: HashMap<TypeId, FQTypeId>,
}

impl Scope {
    pub fn close(self) -> Closed {
        Closed {
            scope: Rc::new(self),
        }
    }

    pub fn builtin() -> Self {
        let mut scope: Scope = Default::default();
        scope.add_builtin_types();
        scope
    }

    fn add_fq_kind(&mut self, id: FQTypeId, kind: Kind) -> Result<(), ScopeError> {
        if self.fq_kinds.contains_key(&id) {
            Err(ScopeError::DuplicateFQTypeId(id))
        } else {
            self.fq_kinds.insert(id, kind);
            Ok(())
        }
    }

    fn add_local_kind(&mut self, id: TypeId, fq: FQTypeId) -> Result<(), ScopeError> {
        if !self.fq_kinds.contains_key(&fq) {
            Err(ScopeError::UnknownFQTypeId(fq))
        } else {
            self.local_kinds.insert(id, fq);
            Ok(())
        }
    }

    fn add_std_type(&mut self, name: &ArcStr, tipo: Type) {
        let id = TypeId::new(name.clone()).unwrap();
        let fq = FQTypeId::std(id.clone());
        self.add_fq_kind(fq.clone(), Kind::Type(tipo)).unwrap();
        self.add_local_kind(id, fq).unwrap();
    }

    fn add_builtin_types(&mut self) {
        self.add_std_type(&BOOLEAN, Type::boolean());
        self.add_std_type(&TRUE, Type::boolean_true());
        self.add_std_type(&FALSE, Type::boolean_false());
        self.add_std_type(&INTEGER, Type::integer());
        self.add_std_type(&NONE, Type::none());
    }
}

#[derive(Debug)]
pub enum ScopeError {
    DuplicateFQTypeId(FQTypeId),
    UnknownFQTypeId(FQTypeId),
}

impl crate::error::Error for ScopeError {}
