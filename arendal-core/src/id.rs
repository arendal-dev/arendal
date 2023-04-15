use crate::ArcStr;
use std::{
    fmt::{self, Write},
    sync::Arc,
};

// We use SHA3-256 hashes as ids when using content-adressing
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RawId {
    bytes: [u8; 32],
}

impl RawId {
    pub fn as_string(&self) -> String {
        data_encoding::HEXLOWER.encode(&self.bytes)
    }

    pub fn as_arcstr(&self) -> ArcStr {
        self.as_string().into()
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

impl From<u64> for RawId {
    fn from(value: u64) -> Self {
        let mut bytes = [0u8; 32];
        let le = value.to_le_bytes();
        for i in 0..le.len() {
            bytes[i] = le[i];
        }
        RawId { bytes }
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

    pub fn as_arcstr(&self) -> ArcStr {
        self.id.as_arcstr()
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
