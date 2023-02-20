use super::ArcStr;

#[derive(Debug, Clone)]
pub struct Loc {
    _inner: Inner,
}

impl Loc {
    pub fn input(input: ArcStr, pos: usize) -> Self {
        Loc {
            _inner: Inner::Input(input, pos),
        }
    }

    pub fn none() -> Self {
        Loc {
            _inner: Inner::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner {
    None,
    Input(ArcStr, usize),
}
