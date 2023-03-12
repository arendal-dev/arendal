use im::HashMap;

use crate::id::{FQTypeId, TypeId};
use crate::types::Type;
use crate::{literal, ArcStr};

pub(crate) static BOOLEAN: ArcStr = literal!("Boolean");
pub(crate) static TRUE: ArcStr = literal!("True");
pub(crate) static FALSE: ArcStr = literal!("False");
pub(crate) static INTEGER: ArcStr = literal!("Integer");
pub(crate) static NONE: ArcStr = literal!("None");

#[derive(Debug, Clone)]
enum Kind {
    Type(Type),
}

#[derive(Debug, Clone, Default)]
pub struct Names {
    fq_kinds: HashMap<FQTypeId, Kind>,
    local_kinds: HashMap<TypeId, FQTypeId>,
}

impl Names {
    pub fn builtin() -> Self {
        let mut names: Names = Default::default();
        names.add_builtin_types();
        names
    }

    fn add_fq_kind(&mut self, id: FQTypeId, kind: Kind) -> Result<(), NamesError> {
        if self.fq_kinds.contains_key(&id) {
            Err(NamesError::DuplicateFQTypeId(id))
        } else {
            self.fq_kinds.insert(id, kind);
            Ok(())
        }
    }

    fn add_local_kind(&mut self, id: TypeId, fq: FQTypeId) -> Result<(), NamesError> {
        if !self.fq_kinds.contains_key(&fq) {
            Err(NamesError::UnknownFQTypeId(fq))
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
pub enum NamesError {
    DuplicateFQTypeId(FQTypeId),
    UnknownFQTypeId(FQTypeId),
}

impl crate::error::Error for NamesError {}
