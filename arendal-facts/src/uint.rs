// Internal fact implementation
#[derive(Debug, Clone)]
enum IFact {
    Empty,
    Any,
    LT(i64),
    Eq(i64),
    GT(i64),
    And(Box<IFact>, Box<IFact>),
    Or(Box<IFact>, Box<IFact>),
}

use self::IFact::*;
use super::Proof;
use super::Proof::*;
use core::panic;
use std::{
    borrow::Borrow,
    cmp::{max, min, Ordering},
};

impl PartialOrd for IFact {
    fn partial_cmp(&self, other: &IFact) -> Option<Ordering> {
        match self {
            Empty => match other {
                Empty => Some(Ordering::Equal),
                _ => None,
            },
            Any => match other {
                Empty => None,
                Any => Some(Ordering::Equal),
                _ => Some(Ordering::Greater),
            },
            _ => self.proof_based_cmp(other),
        }
    }
}

impl PartialEq for IFact {
    fn eq(&self, other: &IFact) -> bool {
        match self {
            Empty => match other {
                Empty => true,
                _ => self.proof_based_eq(other),
            },
            Any => match other {
                Any => true,
                _ => self.proof_based_eq(other),
            },
            LT(v1) => match other {
                LT(v2) => v1 == v2,
                _ => self.proof_based_eq(other),
            },
            Eq(v1) => match other {
                Eq(v2) => v1 == v2,
                _ => self.proof_based_eq(other),
            },
            GT(v1) => match other {
                GT(v2) => v1 == v2,
                _ => self.proof_based_eq(other),
            },
            _ => self.proof_based_eq(other),
        }
    }
}

fn lte(x: i64) -> IFact {
    LT(x + 1)
}

fn gte(x: i64) -> IFact {
    GT(x - 1)
}

fn and(a: IFact, b: IFact) -> IFact {
    And(Box::new(a), Box::new(b))
}

fn or(a: IFact, b: IFact) -> IFact {
    Or(Box::new(a), Box::new(b))
}

impl IFact {
    fn proves(&self, other: &IFact) -> Proof<IFact> {
        match self {
            Empty => match other {
                Empty => Proved,
                _ => Contradiction(Empty),
            },
            Any => match other {
                Empty => Contradiction(Any),
                Any => Proved,
                _ => CannotProve,
            },
            LT(a) => match other {
                Empty => Contradiction(self.clone()),
                Any => CannotProve,
                LT(b) => {
                    if a <= b {
                        Proved
                    } else {
                        CannotProve
                    }
                }
                Eq(_) | GT(_) => Contradiction(self.clone()),
                And(f1, f2) => self.proves_both(f1.borrow(), f2.borrow()),
                Or(f1, f2) => self.proves_any(f1.borrow(), f2.borrow()),
            },
            Eq(a) => match other {
                Empty => Contradiction(self.clone()),
                Any => Proved,
                Eq(b) => {
                    if a == b {
                        Proved
                    } else {
                        Contradiction(self.clone())
                    }
                }
                LT(_) | GT(_) => Contradiction(self.clone()),
                And(f1, f2) => self.proves_both(f1.borrow(), f2.borrow()),
                Or(f1, f2) => self.proves_any(f1.borrow(), f2.borrow()),
            },
            GT(a) => match other {
                Empty => Contradiction(self.clone()),
                Any => Proved,
                GT(b) => {
                    if a >= b {
                        Proved
                    } else {
                        CannotProve
                    }
                }
                Eq(_) | LT(_) => Contradiction(self.clone()),
                And(f1, f2) => self.proves_both(f1.borrow(), f2.borrow()),
                Or(f1, f2) => self.proves_any(f1.borrow(), f2.borrow()),
            },
            And(f1, f2) => {
                let p1 = f1.proves(other);
                let p2 = f2.proves(other);
                match p1 {
                    Proved => Proved,
                    CannotProve => p2,
                    Contradiction(c1) => match p2 {
                        Proved => Proved,
                        CannotProve => Contradiction(c1),
                        Contradiction(c2) => Contradiction(c1.and(c2)),
                    },
                }
            }
            Or(f1, f2) => {
                let p1 = f1.proves(other);
                let p2 = f2.proves(other);
                match p1 {
                    Proved => match p2 {
                        Proved => Proved,
                        _ => p2,
                    },
                    CannotProve => match p2 {
                        Proved | CannotProve => p1,
                        _ => p2,
                    },
                    Contradiction(c1) => match p2 {
                        Proved | CannotProve => Contradiction(c1),
                        Contradiction(c2) => Contradiction(c1.or(c2)),
                    },
                }
            }
        }
    }

    fn proves_both(&self, f1: &IFact, f2: &IFact) -> Proof<IFact> {
        let p1 = self.proves(f1);
        let p2 = self.proves(f2);
        match p1 {
            Proved => p2,
            CannotProve => match p2 {
                Proved | CannotProve => CannotProve,
                _ => p2,
            },
            _ => p1,
        }
    }

    fn proves_any(&self, f1: &IFact, f2: &IFact) -> Proof<IFact> {
        let p1 = self.proves(f1);
        let p2 = self.proves(f2);
        match p1 {
            Proved => Proved,
            CannotProve => match p2 {
                Proved => Proved,
                CannotProve | Contradiction(_) => CannotProve,
            },
            Contradiction(c1) => match p2 {
                Proved => Proved,
                CannotProve => CannotProve,
                Contradiction(c2) => Contradiction(c1.and(c2)),
            },
        }
    }

    fn proof_based_cmp(&self, other: &IFact) -> Option<Ordering> {
        match self.proves(other) {
            Proved => match other.proves(self) {
                Proved => Some(Ordering::Equal),
                CannotProve => Some(Ordering::Less),
                _ => None,
            },
            CannotProve => match other.proves(self) {
                Proved => Some(Ordering::Greater),
                _ => None,
            },
            _ => None,
        }
    }

    fn proof_based_eq(&self, other: &IFact) -> bool {
        match self.proof_based_cmp(other) {
            Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    fn or(self, other: IFact) -> IFact {
        match self {
            Empty => other,
            Any => Any,
            LT(a) => match other {
                Empty => self,
                Any => Any,
                LT(b) => LT(max(a, b)),
                Eq(b) => {
                    if b < a {
                        self
                    } else if b == a {
                        lte(a)
                    } else {
                        or(self, other)
                    }
                }
                GT(b) => {
                    if b <= a {
                        Any
                    } else {
                        or(self, other)
                    }
                }
                _ => self.proof_based_or(other),
            },
            Eq(a) => match other {
                Empty => self,
                Any => Any,
                LT(b) => {
                    if a < b {
                        other
                    } else if a == b {
                        lte(a)
                    } else {
                        or(self, other)
                    }
                }
                Eq(b) => {
                    if a == b {
                        self
                    } else {
                        or(self, other)
                    }
                }
                GT(b) => {
                    if b < a {
                        other
                    } else if a == b {
                        gte(a)
                    } else {
                        or(self, other)
                    }
                }
                _ => self.proof_based_or(other),
            },
            GT(a) => match other {
                Empty => self,
                Any => Any,
                LT(b) => {
                    if a <= b {
                        Any
                    } else {
                        or(self, other)
                    }
                }
                Eq(b) => {
                    if b > a {
                        self
                    } else if a == b {
                        gte(a)
                    } else {
                        or(self, other)
                    }
                }
                GT(b) => GT(min(a, b)),
                _ => self.proof_based_or(other),
            },
            _ => self.proof_based_or(other),
        }
    }

    fn proof_based_or(self, other: IFact) -> IFact {
        match self.partial_cmp(&other) {
            Some(Ordering::Less) => other,
            Some(_) => self,
            _ => or(self, other),
        }
    }

    fn and(self, other: IFact) -> IFact {
        self.proof_based_and(other)
    }

    fn proof_based_and(self, other: IFact) -> IFact {
        match self.partial_cmp(&other) {
            Some(Ordering::Greater) => other,
            Some(_) => self,
            _ => and(self, other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IFact::*;
    use super::{and, gte, lte, or, IFact};

    fn and_range(min: i64, max: i64) -> IFact {
        assert!(max > min);
        and(gte(min), lte(max))
    }

    fn or_range(min: i64, max: i64) -> IFact {
        assert!(max > min);
        or(lte(min), gte(max))
    }

    fn reverse(fact: IFact) -> IFact {
        match fact {
            And(f1, f2) => And(f2, f1),
            Or(f1, f2) => Or(f2, f1),
            _ => fact,
        }
    }

    //#[test]
    fn test_eq() {
        // Same variant
        assert_eq!(Any, Any);
        assert_eq!(LT(2), LT(2));
        assert_ne!(LT(2), LT(4));
        assert_eq!(Eq(2), Eq(2));
        assert_ne!(Eq(2), Eq(4));
        assert_eq!(GT(2), GT(2));
        assert_ne!(GT(2), GT(4));
        // Different variant
        assert_ne!(Any, LT(2));
        assert_ne!(Any, Eq(2));
        assert_ne!(Any, GT(2));
        assert_ne!(Any, and_range(5, 10));
        assert_ne!(Any, or_range(5, 10));
        assert_ne!(LT(2), Eq(2));
        assert_ne!(LT(2), GT(2));
        assert_ne!(LT(2), and_range(5, 10));
        assert_ne!(LT(2), or_range(5, 10));
        assert_ne!(Eq(2), GT(2));
        assert_ne!(Eq(2), and_range(5, 10));
        assert_ne!(Eq(2), or_range(5, 10));
        assert_ne!(GT(2), and_range(5, 10));
        assert_ne!(GT(2), or_range(5, 10));
        assert_ne!(and_range(5, 10), or_range(5, 10));
        // Operations are commutative
        assert_eq!(and_range(5, 10), reverse(and_range(5, 10)));
        assert_eq!(or_range(5, 10), reverse(or_range(5, 10)));
    }
}
