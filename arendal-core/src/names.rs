use std::collections::HashMap;

use crate::error::{Error, Errors, Loc, Result};
use crate::identifier::{FQTypeId, Identifier, TypeIdentifier};
use crate::types::Type;
use crate::{literal, ArcStr};

// TODO: remove once manifests are available
static BOOLEAN: ArcStr = literal!("Boolean");
static TRUE: ArcStr = literal!("True");
static FALSE: ArcStr = literal!("False");
static INTEGER: ArcStr = literal!("Integer");
static NONE: ArcStr = literal!("None");

#[derive(Debug, Clone)]
enum Kind {
    Type(Type),
}

#[derive(Debug, Clone, Default)]
struct ValScope {
    vals: HashMap<Identifier, Type>,
}

impl ValScope {
    fn get(&self, id: &Identifier) -> Option<Type> {
        self.vals.get(id).cloned()
    }

    fn add(&mut self, loc: Loc, id: Identifier, tipo: Type) -> Result<()> {
        if self.vals.contains_key(&id) {
            return Errors::err(loc, NamesError::DuplicateVal(id));
        }
        self.vals.insert(id, tipo);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Names {
    fq_kinds: HashMap<FQTypeId, Kind>,
    local_kinds: HashMap<TypeIdentifier, FQTypeId>,
    val_scopes: Vec<ValScope>,
}

impl Default for Names {
    fn default() -> Self {
        Names {
            fq_kinds: Default::default(),
            local_kinds: Default::default(),
            val_scopes: vec![Default::default()],
        }
    }
}

impl Names {
    pub fn builtin() -> Self {
        let mut names: Names = Default::default();
        names.add_builtin_types();
        names
    }

    fn add_fq_kind(&mut self, id: FQTypeId, kind: Kind) -> Result<()> {
        if self.fq_kinds.contains_key(&id) {
            Errors::err(Loc::none(), NamesError::DuplicateFQTypeId(id))
        } else {
            self.fq_kinds.insert(id, kind);
            Ok(())
        }
    }

    fn add_local_kind(&mut self, id: TypeIdentifier, fq: FQTypeId) -> Result<()> {
        if !self.fq_kinds.contains_key(&fq) {
            Errors::err(Loc::none(), NamesError::UnknownFQTypeId(fq))
        } else {
            self.local_kinds.insert(id, fq);
            Ok(())
        }
    }

    fn add_std_type(&mut self, name: &ArcStr, tipo: Type) {
        let id = TypeIdentifier::new(name.clone()).unwrap();
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

    pub fn push_val_scope(&mut self) -> usize {
        self.val_scopes.push(Default::default());
        self.val_scopes.len()
    }

    pub fn pop_val_scope(&mut self, key: usize) {
        assert!(
            key > 1 && key == self.val_scopes.len(),
            "Removing wrong val scope"
        );
        self.val_scopes.pop();
    }

    pub fn add_val(&mut self, loc: Loc, id: Identifier, tipo: Type) -> Result<()> {
        self.val_scopes.last_mut().unwrap().add(loc, id, tipo)
    }

    pub fn get_val(&self, id: &Identifier) -> Option<Type> {
        let mut i = self.val_scopes.len();
        while i > 0 {
            let result = self.val_scopes[i - 1].get(id);
            if result.is_some() {
                return result;
            }
            i = i - 1;
        }
        None
    }
}

#[derive(Debug)]
pub enum NamesError {
    DuplicateFQTypeId(FQTypeId),
    UnknownFQTypeId(FQTypeId),
    DuplicateVal(Identifier),
}

impl Error for NamesError {}
