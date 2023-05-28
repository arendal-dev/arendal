mod builtin;

use std::fmt;
use std::sync::Arc;

use im::HashMap;

use crate::error::{Error, Errors, Result, L};
use crate::symbol::FQType;
use crate::visibility::{Visibility, V};

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    None,
    True,
    False,
    Boolean,
    Integer,
    Singleton(Singleton),
    Tuple(Arc<Tuple>),
}

// We need some kind of indirection to reference types
// to allow for recursive type definitions.
#[derive(Clone, PartialEq, Eq)]
enum Ref {
    Type(Type),
    Symbol(FQType),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Singleton {
    symbol: FQType,
}

impl fmt::Display for Singleton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.symbol.fmt(f)
    }
}

impl fmt::Debug for Singleton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Tuple {
    symbol: FQType,
    types: Vec<Ref>,
}

impl Type {
    pub fn fq(&self) -> FQType {
        match self {
            Self::None => FQType::None,
            Self::True => FQType::True,
            Self::False => FQType::False,
            Self::Boolean => FQType::Boolean,
            Self::Integer => FQType::Integer,
            Self::Singleton(s) => s.symbol.clone(),
            Self::Tuple(t) => t.symbol.clone(),
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Self::True | Self::False | Self::Boolean => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        *self == Self::Integer
    }

    pub fn is_singleton(&self) -> bool {
        match self {
            Type::None | Type::True | Type::False | Type::Singleton(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fq().fmt(f)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

// We need this level of indirection to add refinements in the future.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    fq_type: FQType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeDfn {
    Singleton,
    Tuple(Vec<TypeRef>),
}

pub(crate) type LVTypeDfn = L<V<TypeDfn>>;
pub(crate) type TypeDfnMap = HashMap<FQType, LVTypeDfn>;

type TypeMap = HashMap<FQType, V<Type>>;

#[derive(Debug, Clone)]
pub(crate) struct Types {
    types: TypeMap,
}

impl Default for Types {
    fn default() -> Self {
        Types {
            types: builtin::get_builtin_types(),
        }
    }
}

impl Types {
    pub(crate) fn get(&self, symbol: &FQType) -> Option<&V<Type>> {
        self.types.get(symbol)
    }

    pub(crate) fn contains(&self, symbol: &FQType) -> bool {
        self.types.contains_key(symbol)
    }

    fn add(&mut self, fq: &FQType, visibility: Visibility, tipo: Type) {
        self.types.insert(fq.clone(), visibility.wrap(tipo));
    }

    fn add_singleton(&mut self, fq: &FQType, visibility: Visibility) {
        self.add(
            fq,
            visibility,
            Type::Singleton(Singleton { symbol: fq.clone() }),
        );
    }

    pub(crate) fn add_types(&self, candidates: &TypeDfnMap) -> Result<Types> {
        let mut result = self.clone();
        if candidates.is_empty() {
            return Ok(result);
        }
        let mut errors = Errors::default();
        for (fq, lvdfn) in candidates {
            if self.types.contains_key(fq) {
                errors.add(lvdfn.error(Error::DuplicateType(fq.clone())));
            } else {
                match &lvdfn.it.it {
                    TypeDfn::Singleton => result.add_singleton(fq, lvdfn.it.visibility),
                    _ => todo!(),
                }
            }
        }
        errors.to_result(result)
    }
}
