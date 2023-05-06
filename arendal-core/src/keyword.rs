use phf::phf_map;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Pkg,
    Let,
    If,
    Then,
    Else,
    Type,
}

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "pkg" => Keyword::Pkg,
    "let" => Keyword::Let,
    "if" => Keyword::If,
    "then" => Keyword::Then,
    "else" => Keyword::Else,
    "type" => Keyword::Type,
};

impl Keyword {
    pub fn parse(keyword: &str) -> Option<Self> {
        KEYWORDS.get(keyword).cloned()
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Pkg => "pkg",
            Self::Let => "let",
            Self::If => "if",
            Self::Then => "then",
            Self::Else => "else",
            Self::Type => "type",
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
