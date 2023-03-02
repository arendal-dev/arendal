use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner {
    Boolean,
    True,
    False,
    Integer,
    None,
    Some(Type),
}

impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Type {
    inner: Rc<Inner>,
}

impl Type {
    fn new(tipo: Inner) -> Self {
        Type {
            inner: Rc::new(tipo),
        }
    }

    pub fn integer() -> Self {
        Self::new(Inner::Integer)
    }

    pub fn is_integer(&self) -> bool {
        matches!(*self.inner, Inner::Integer)
    }

    pub fn boolean() -> Self {
        Self::new(Inner::Boolean)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(*self.inner, Inner::Boolean | Inner::True | Inner::False)
    }

    pub fn boolean_true() -> Self {
        Self::new(Inner::True)
    }

    pub fn is_boolean_true(&self) -> bool {
        matches!(*self.inner, Inner::True)
    }

    pub fn boolean_false() -> Self {
        Self::new(Inner::False)
    }

    pub fn is_boolean_false(&self) -> bool {
        matches!(*self.inner, Inner::False)
    }

    pub fn none() -> Self {
        Self::new(Inner::None)
    }

    pub fn is_none(&self) -> bool {
        matches!(*self.inner, Inner::None)
    }

    pub fn some(tipo: Type) -> Self {
        Self::new(Inner::Some(tipo))
    }

    pub fn is_some(&self) -> bool {
        matches!(*self.inner, Inner::Some(_))
    }

    pub fn is_option(&self) -> bool {
        matches!(*self.inner, Inner::None | Inner::Some(_))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner.as_ref())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::Type;
    #[test]
    fn eq() {
        let r = Type::integer();
        assert_eq!(r, r.clone());
    }
}
