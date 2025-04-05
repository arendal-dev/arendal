use std::{
    fmt::{self, Write},
    sync::Arc,
};

#[derive(Clone, PartialEq, Eq, Hash)]
struct RawId {
    bytes: [u8; 32],
}

impl RawId {
    pub fn as_string(&self) -> String {
        data_encoding::HEXLOWER.encode(&self.bytes)
    }
}

impl fmt::Display for RawId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_string().as_str())
    }
}

impl fmt::Debug for RawId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Id(")?;
        fmt::Display::fmt(self, f)?;
        f.write_char(')')
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id {
    id: Arc<RawId>,
}

impl Id {
    pub fn as_string(&self) -> String {
        self.id.as_string()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}
