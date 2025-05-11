use ast::Payload;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Unit,
    True,
    False,
    Integer, // Temporary
}

impl Payload for Type {}
