/// Internal representation of a fact
#[derive(PartialEq, Eq, Debug)]
pub enum Fact {
    Empty,
    True,
    False,
    Any,
}

use self::Fact::*;
use super::Proof;
use super::Proof::*;

impl Fact {
    pub fn or(self, other: Fact) -> Fact {
        match self {
            Empty => other,
            True => match other {
                Empty | True => True,
                _ => Any,
            },
            False => match other {
                Empty | False => False,
                _ => Any,
            },
            Any => Any,
        }
    }

    pub fn and(self, other: Fact) -> Fact {
        match self {
            Empty => Empty,
            True => match other {
                Empty | False => Empty,
                True | Any => True,
            },
            False => match other {
                Empty | True => Empty,
                False | Any => False,
            },
            Any => match other {
                Empty => Empty,
                _ => other,
            },
        }
    }

    pub fn proves(self, other: Fact) -> Proof<Fact> {
        match self {
            Empty => match other {
                Empty => Proved,
                _ => Contradiction(Empty),
            },
            True => match other {
                True | Any => Proved,
                Empty | False => Contradiction(True),
            },
            False => match other {
                False | Any => Proved,
                Empty | True => Contradiction(False),
            },
            Any => match other {
                Empty => Contradiction(Any),
                True | False => CannotProve,
                Any => Proved,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Fact::*;
    use super::Proof::*;

    #[test]
    fn or() {
        assert_eq!(Empty.or(Empty), Empty);
        assert_eq!(Empty.or(True), True);
        assert_eq!(Empty.or(False), False);
        assert_eq!(Empty.or(Any), Any);
        assert_eq!(True.or(Empty), True);
        assert_eq!(False.or(Empty), False);
        assert_eq!(Any.or(Empty), Any);
        assert_eq!(True.or(True), True);
        assert_eq!(False.or(False), False);
        assert_eq!(True.or(False), Any);
        assert_eq!(False.or(True), Any);
        assert_eq!(True.or(Any), Any);
        assert_eq!(False.or(Any), Any);
        assert_eq!(Any.or(True), Any);
        assert_eq!(Any.or(False), Any);
    }

    #[test]
    fn and() {
        assert_eq!(Empty.and(Empty), Empty);
        assert_eq!(Empty.and(True), Empty);
        assert_eq!(Empty.and(False), Empty);
        assert_eq!(Empty.and(Any), Empty);
        assert_eq!(True.and(Empty), Empty);
        assert_eq!(False.and(Empty), Empty);
        assert_eq!(Any.and(Empty), Empty);
        assert_eq!(True.and(True), True);
        assert_eq!(False.and(False), False);
        assert_eq!(True.and(False), Empty);
        assert_eq!(False.and(True), Empty);
        assert_eq!(True.and(Any), True);
        assert_eq!(False.and(Any), False);
        assert_eq!(Any.and(True), True);
        assert_eq!(Any.and(False), False);
    }

    #[test]
    fn proofs() {
        assert_eq!(Empty.proves(Empty), Proved);
        assert_eq!(Empty.proves(True), Contradiction(Empty));
        assert_eq!(Empty.proves(False), Contradiction(Empty));
        assert_eq!(Empty.proves(Any), Contradiction(Empty));
        assert_eq!(True.proves(Empty), Contradiction(True));
        assert_eq!(False.proves(Empty), Contradiction(False));
        assert_eq!(Any.proves(Empty), Contradiction(Any));
        assert_eq!(True.proves(True), Proved);
        assert_eq!(True.proves(False), Contradiction(True));
        assert_eq!(True.proves(Any), Proved);
        assert_eq!(False.proves(False), Proved);
        assert_eq!(False.proves(True), Contradiction(False));
        assert_eq!(False.proves(Any), Proved);
        assert_eq!(Any.proves(Any), Proved);
        assert_eq!(Any.proves(True), CannotProve);
        assert_eq!(Any.proves(False), CannotProve);
    }
}
