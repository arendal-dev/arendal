use phf::phf_map;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TSymbols {
    None,
    True,
    False,
    Boolean,
    Integer,
}

static T_SYMBOLS: phf::Map<&'static str, TSymbols> = phf_map! {
    "None" => TSymbols::None,
    "True" => TSymbols::True,
    "False" => TSymbols::False,
    "Boolean" => TSymbols::Boolean,
    "Integer" => TSymbols::Integer,
};

impl TSymbols {
    pub(crate) fn parse(keyword: &str) -> Option<Self> {
        T_SYMBOLS.get(keyword).cloned()
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::True => "True",
            Self::False => "False",
            Self::Boolean => "Boolean",
            Self::Integer => "Integer",
        }
    }
}

impl fmt::Display for TSymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for TSymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TSymbols({})", self.as_str())
    }
}
