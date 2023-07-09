use std::fmt;
use std::sync::Arc;

use im::HashMap;
use num::Integer;

use crate::error::{Error, Errors, Loc, Result, L};
use crate::symbol::{self, FQType};
use crate::visibility::{Visibility, V};

#[derive(Clone, PartialEq, Eq)]
pub struct Type {
    data: TypeData,
}

#[derive(Clone, PartialEq, Eq)]
enum TypeData {
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
    pub fn type_none() -> Type {
        Type {
            data: TypeData::None,
        }
    }

    pub fn type_true() -> Type {
        Type {
            data: TypeData::True,
        }
    }

    pub fn type_false() -> Type {
        Type {
            data: TypeData::False,
        }
    }

    pub fn type_boolean() -> Type {
        Type {
            data: TypeData::Boolean,
        }
    }

    pub fn type_integer() -> Type {
        Type {
            data: TypeData::Integer,
        }
    }

    pub fn fq(&self) -> FQType {
        match &self.data {
            TypeData::None => symbol::FQ_NONE.clone(),
            TypeData::True => symbol::FQ_TRUE.clone(),
            TypeData::False => symbol::FQ_FALSE.clone(),
            TypeData::Boolean => symbol::FQ_BOOLEAN.clone(),
            TypeData::Integer => symbol::FQ_INTEGER.clone(),
            TypeData::Singleton(s) => s.symbol.clone(),
            TypeData::Tuple(t) => t.symbol.clone(),
        }
        .clone()
    }

    pub fn is_none(&self) -> bool {
        self.data == TypeData::None
    }

    pub fn is_true(&self) -> bool {
        self.data == TypeData::True
    }

    pub fn is_false(&self) -> bool {
        self.data == TypeData::False
    }

    pub fn is_boolean(&self) -> bool {
        match self.data {
            TypeData::True | TypeData::False | TypeData::Boolean => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        self.data == TypeData::Integer
    }

    pub fn is_singleton(&self) -> bool {
        match self.data {
            TypeData::None | TypeData::True | TypeData::False | TypeData::Singleton(_) => true,
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

#[derive(Clone, PartialEq, Eq)]
pub struct Value {
    tipo: Type,
    val: Val,
}

#[derive(Clone, PartialEq, Eq)]
enum Val {
    Singleton,
    Integer(Integer),
}

impl Value {
    pub fn get_type(&self) -> Type {
        self.tipo.clone()
    }

    pub fn as_integer(self, loc: &Loc) -> Result<Integer> {
        match self.val {
            Val::Integer(v) => Ok(v),
            _ => self.type_mismatch(loc, Type::type_integer()),
        }
    }

    pub fn as_boolean(self, loc: &Loc) -> Result<bool> {
        match self.tipo.data {
            TypeData::True => Ok(true),
            TypeData::False => Ok(false),
            _ => self.type_mismatch(loc, Type::type_boolean()),
        }
    }

    fn type_mismatch<T>(&self, loc: &Loc, expected: Type) -> Result<T> {
        loc.err(Error::type_mismatch(expected, self.get_type()))
    }

    fn v_builtin_singleton(data: TypeData) -> Value {
        Value {
            tipo: Type { data },
            val: Val::Singleton,
        }
    }

    pub fn v_none() -> Value {
        Self::v_builtin_singleton(TypeData::None)
    }

    pub fn v_true() -> Value {
        Self::v_builtin_singleton(TypeData::True)
    }

    pub fn v_false() -> Value {
        Self::v_builtin_singleton(TypeData::False)
    }

    pub fn v_bool(value: bool) -> Value {
        if value {
            Self::v_true()
        } else {
            Self::v_false()
        }
    }

    pub fn v_integer(loc: &Loc, tipo: Type, value: Integer) -> Result<Value> {
        if tipo.is_integer() {
            Ok(Value {
                tipo,
                val: Val::Integer(value),
            })
        } else {
            loc.err(Error::SingletonExpected(tipo))
        }
    }

    pub fn v_singleton(loc: &Loc, tipo: Type) -> Result<Value> {
        if tipo.is_singleton() {
            Ok(Value {
                tipo,
                val: Val::Singleton,
            })
        } else {
            loc.err(Error::SingletonExpected(tipo))
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.val {
            Val::Integer(value) => value.fmt(f),
            Val::Singleton => self.tipo.fmt(f),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Types {
    types: TypeMap,
}

impl Types {
    fn export(&mut self, tipo: Type) {
        self.types
            .insert(tipo.fq(), Visibility::Exported.wrap(tipo));
    }

    pub(crate) fn get(&self, symbol: &FQType) -> Option<&V<Type>> {
        self.types.get(symbol)
    }

    pub(crate) fn contains(&self, symbol: &FQType) -> bool {
        self.types.contains_key(symbol)
    }

    fn add(&mut self, fq: &FQType, visibility: Visibility, data: TypeData) {
        self.types
            .insert(fq.clone(), visibility.wrap(Type { data }));
    }

    fn add_singleton(&mut self, fq: &FQType, visibility: Visibility) {
        self.add(
            fq,
            visibility,
            TypeData::Singleton(Singleton { symbol: fq.clone() }),
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

impl Default for Types {
    fn default() -> Self {
        let mut types = Types {
            types: Default::default(),
        };
        types.export(Type::type_none());
        types.export(Type::type_true());
        types.export(Type::type_false());
        types.export(Type::type_boolean());
        types.export(Type::type_integer());
        types
    }
}
