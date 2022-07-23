// Internal fact implementation
#[derive(Debug, Clone)]
enum IFact {
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
use std::cmp::{max, min};

impl PartialEq for IFact {
    fn eq(&self, other: &IFact) -> bool {
        match self {
            Any => match other {
                Any => true,
                _ => false,
            },
            LT(v1) => match other {
                LT(v2) => v1 == v2,
                _ => false,
            },
            Eq(v1) => match other {
                Eq(v2) => v1 == v2,
                _ => false,
            },
            GT(v1) => match other {
                GT(v2) => v1 == v2,
                _ => false,
            },
            And(f11, f12) => match other {
                And(f21, f22) => (f11 == f21 && f12 == f22) || (f11 == f22 && f12 == f21),
                _ => false,
            },
            Or(f11, f12) => match other {
                Or(f21, f22) => (f11 == f21 && f12 == f22) || (f11 == f22 && f12 == f21),
                _ => false,
            },
        }
    }
}

impl std::cmp::Eq for IFact {}

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
    fn proves(self, other: IFact) -> Proof<IFact> {
        match self {
            Any => Proved,
            LT(a) => match other {
                Any => Proved,
                LT(b) => {
                    if a <= b {
                        Proved
                    } else {
                        Contradiction(self)
                    }
                }
                Eq(_) | GT(_) => Contradiction(self),
                And(f1, f2) => panic!("TODO"),
                Or(f1, f2) => panic!("TODO"),
            },
            _ => panic!("TODO"),
        }
    }

    fn proves_both(self, f1: IFact, f2: IFact) -> Proof<IFact> {
        let p1 = self.clone().proves(f1);
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

    fn or(self, other: IFact) -> IFact {
        if (self == Any || other == Any) {
            Any
        } else if (self == other) {
            self
        } else {
            match self {
                LT(a) => match other {
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
                    _ => panic!("TODO"),
                },
                _ => panic!("TODO"),
            }
        }
    }

    fn and(self, other: IFact) -> Result<IFact, IFact> {
        match self {
            _ => Err(self),
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

    #[test]
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
