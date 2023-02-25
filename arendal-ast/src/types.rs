use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Boolean,
    True,
    False,
    Integer,
    None,
    Some(TypeRef),
}

impl Type {
    pub fn to_ref(self) -> TypeRef {
        TypeRef::new(self)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TypeRef {
    inner: Rc<Type>,
}

impl TypeRef {
    fn new(tipo: Type) -> Self {
        TypeRef {
            inner: Rc::new(tipo),
        }
    }
}

impl std::convert::AsRef<Type> for TypeRef {
    fn as_ref(&self) -> &Type {
        self.inner.as_ref()
    }
}

impl std::ops::Deref for TypeRef {
    type Target = Type;

    fn deref(&self) -> &Type {
        self.inner.deref()
    }
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl fmt::Debug for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::Type;
    #[test]
    fn eq() {
        let r = Type::Integer.to_ref();
        assert_eq!(r, r.clone());
    }
}
