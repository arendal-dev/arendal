use crate::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
    Module,
    Package,
    Exported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeIdKind {
    Type(Type)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IdKind {
    Val(Type)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Target<K> {
    visiblity: Visibility,
    kind: K,
}