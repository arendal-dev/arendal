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
    Singleton(FQType),
    NamedTuple(Arc<NamedTuple>),
    AnonTuple(Arc<Tuple>),
}

impl TypeData {
    fn named_tuple(symbol: &FQType, types: Vec<TypeRef>) -> Self {
        TypeData::NamedTuple(Arc::new(NamedTuple {
            symbol: symbol.clone(),
            types: Tuple { types },
        }))
    }
}

// We need some kind of indirection to reference types
// to allow for recursive type definitions.
#[derive(Clone, PartialEq, Eq)]
enum TypeRef {
    Type(Type),
    Symbol(FQType),
}

#[derive(Clone, PartialEq, Eq)]
struct Tuple {
    types: Vec<TypeRef>,
}

#[derive(Clone, PartialEq, Eq)]
struct NamedTuple {
    symbol: FQType,
    types: Tuple,
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

    pub fn is_anonymous(&self) -> bool {
        false
    }

    pub fn fq(&self) -> Option<FQType> {
        Some(
            match &self.data {
                TypeData::None => &symbol::FQ_NONE,
                TypeData::True => &symbol::FQ_TRUE,
                TypeData::False => &symbol::FQ_FALSE,
                TypeData::Boolean => &symbol::FQ_BOOLEAN,
                TypeData::Integer => &symbol::FQ_INTEGER,
                TypeData::Singleton(fq) => fq,
                TypeData::NamedTuple(t) => &t.symbol,
                TypeData::AnonTuple(_) => return None,
            }
            .clone(),
        )
    }

    pub fn is_builtin(&self) -> bool {
        match &self.data {
            TypeData::None
            | TypeData::True
            | TypeData::False
            | TypeData::Boolean
            | TypeData::Integer => true,
            _ => false,
        }
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
        match self.fq() {
            Some(fq) => fq.fmt(f),
            None => f.write_str("Anonymous - TODO!"),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeDfnRef {
    Symbol(FQType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeDfn {
    Singleton,
    Tuple(Vec<L<TypeDfnRef>>),
}

pub(crate) type LVTypeDfn = L<V<TypeDfn>>;
pub(crate) type TypeDfnMap = HashMap<FQType, LVTypeDfn>;

type TypeMap = HashMap<FQType, V<Type>>;

#[derive(Debug, Clone, Default)]
struct Types {
    types: TypeMap,
}

impl Types {
    fn add(&self, candidates: &TypeDfnMap) -> Result<Types> {
        if candidates.is_empty() {
            return Ok(self.clone());
        }
        let added = NewTypes::get(&self.types, candidates)?;
        Ok(Types {
            types: self.types.clone().union(added),
        })
    }
}

struct NewTypes<'a> {
    types: &'a TypeMap,
    candidates: &'a TypeDfnMap,
    added: TypeMap,
}

impl<'a> NewTypes<'a> {
    fn get(types: &TypeMap, candidates: &TypeDfnMap) -> Result<TypeMap> {
        NewTypes {
            types,
            candidates,
            added: Default::default(),
        }
        .validate()
    }

    fn add(&mut self, fq: &FQType, visibility: Visibility, data: TypeData) {
        self.added
            .insert(fq.clone(), visibility.wrap(Type { data }));
    }

    fn validate(mut self) -> Result<TypeMap> {
        let mut errors = Errors::default();
        for (fq, lvdfn) in self.candidates {
            if let Some(data) = errors.add_result(self.validate_dfn(fq, lvdfn)) {
                self.add(fq, lvdfn.it.visibility, data)
            }
        }
        errors.to_result(self.added)
    }

    fn validate_dfn(&self, fq: &FQType, lvdfn: &LVTypeDfn) -> Result<TypeData> {
        if self.types.contains_key(fq) {
            lvdfn.err(Error::DuplicateType(fq.clone()))
        } else {
            match &lvdfn.it.it {
                TypeDfn::Singleton => Ok(TypeData::Singleton(fq.clone())),
                TypeDfn::Tuple(types) => {
                    if types.is_empty() {
                        Ok(TypeData::None)
                    } else {
                        let mut errors = Errors::default();
                        let mut refs = Vec::<TypeRef>::with_capacity(types.len());
                        for dfnref in types {
                            errors
                                .add_result(self.validate_ref(dfnref))
                                .map(|r| refs.push(r));
                        }
                        errors.to_lazy_result(|| TypeData::named_tuple(fq, refs))
                    }
                }
                _ => todo!(),
            }
        }
    }

    fn validate_ref(&self, dfnref: &L<TypeDfnRef>) -> Result<TypeRef> {
        match &dfnref.it {
            TypeDfnRef::Symbol(s) => {
                if let Some(t) = self.types.get(s) {
                    // TODO: check visibility
                    if t.it.is_builtin() || t.it.is_singleton() {
                        Ok(TypeRef::Type(t.it.clone()))
                    } else {
                        Ok(TypeRef::Symbol(s.clone()))
                    }
                } else if let Some(t) = self.candidates.get(s) {
                    // TODO: check visibility
                    Ok(TypeRef::Symbol(s.clone()))
                } else {
                    dfnref.err(Error::InvalidType) // TODO
                }
            }
        }
    }
}

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
pub(crate) struct Context {
    types: Types,
}

impl Context {
    fn export(&mut self, tipo: Type) {
        self.types
            .types
            .insert(tipo.fq().unwrap(), Visibility::Exported.wrap(tipo));
    }

    pub(crate) fn get(&self, symbol: &FQType) -> Option<&V<Type>> {
        self.types.types.get(symbol)
    }

    pub(crate) fn contains(&self, symbol: &FQType) -> bool {
        self.types.types.contains_key(symbol)
    }

    fn add(&mut self, fq: &FQType, visibility: Visibility, data: TypeData) {
        self.types
            .types
            .insert(fq.clone(), visibility.wrap(Type { data }));
    }

    pub(crate) fn add_types(&self, candidates: &TypeDfnMap) -> Result<Context> {
        if candidates.is_empty() {
            return Ok(self.clone());
        }
        Ok(Context {
            types: self.types.add(candidates)?,
        })
    }
}

impl Default for Context {
    fn default() -> Self {
        let mut types = Context {
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
