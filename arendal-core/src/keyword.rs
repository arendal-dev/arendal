use phf::phf_map;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Val,
}

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "val" => Keyword::Val,
};

impl Keyword {
    pub fn parse(keyword: &str) -> Option<Self> {
        KEYWORDS.get(keyword).cloned()
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Val => "val",
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Keyword({})", self.as_str())
    }
}
