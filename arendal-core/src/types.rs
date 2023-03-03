use std::fmt;
use std::rc::Rc;

use crate::{literal, ArcStr};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner {
    Boolean,
    True,
    False,
    Integer,
    None,
    Some(Type),
    Option(Type),
    Singleton(ArcStr),
}

pub(crate) static BOOLEAN: ArcStr = literal!("Boolean");
pub(crate) static TRUE: ArcStr = literal!("True");
pub(crate) static FALSE: ArcStr = literal!("False");
pub(crate) static INTEGER: ArcStr = literal!("Integer");
pub(crate) static NONE: ArcStr = literal!("None");
pub(crate) static SOME: ArcStr = literal!("Some");
pub(crate) static OPTION: ArcStr = literal!("Option");

impl Inner {
    fn get_name(&self) -> ArcStr {
        match self {
            Inner::Boolean => BOOLEAN.clone(),
            Inner::True => TRUE.clone(),
            Inner::False => FALSE.clone(),
            Inner::Integer => INTEGER.clone(),
            Inner::None => NONE.clone(),
            Inner::Some(t) => format!("Some({})", t).into(),
            Inner::Option(t) => format!("Option({})", t).into(),
            Inner::Singleton(s) => s.clone(),
        }
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

    fn get_name(&self) -> ArcStr {
        self.inner.get_name()
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
        matches!(*self.inner, Inner::None | Inner::Some(_) | Inner::Option(_))
    }

    pub fn is_singleton(&self) -> bool {
        matches!(
            *self.inner,
            Inner::True | Inner::False | Inner::None | Inner::Singleton(_)
        )
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_name().as_str())
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
