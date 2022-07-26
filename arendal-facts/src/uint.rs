// Internal fact implementation
#[derive(Debug, Clone)]
enum IFact {
    Empty,
    Any,
    LT(i64),
    Eq(i64),
    NE(i64),
    GT(i64),
    And(Box<IFact>, Box<IFact>),
    Or(Box<IFact>, Box<IFact>),
}

use self::IFact::*;
use super::Proof;
use super::Proof::*;
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

#[inline]
fn operator_eq(f1: &IFact, f11: &IFact, f12: &IFact, f2: &IFact, f21: &IFact, f22: &IFact) -> bool {
    (f11 == f21 && f12 == f22) || (f11 == f22 && f12 == f21) || f1.proof_based_eq(f2)
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
            NE(v1) => match other {
                NE(v2) => v1 == v2,
                _ => self.proof_based_eq(other),
            },
            GT(v1) => match other {
                GT(v2) => v1 == v2,
                _ => self.proof_based_eq(other),
            },
            And(f11, f12) => match other {
                And(f21, f22) => operator_eq(
                    self,
                    f11.borrow(),
                    f12.borrow(),
                    other,
                    f21.borrow(),
                    f22.borrow(),
                ),
                _ => self.proof_based_eq(other),
            },
            Or(f11, f12) => match other {
                Or(f21, f22) => operator_eq(
                    self,
                    f11.borrow(),
                    f12.borrow(),
                    other,
                    f21.borrow(),
                    f22.borrow(),
                ),
                _ => self.proof_based_eq(other),
            },
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

#[inline]
fn or_lt_eq(flt: IFact, vlt: i64, feq: IFact, veq: i64) -> IFact {
    if veq < vlt {
        flt
    } else if veq == vlt {
        lte(vlt)
    } else {
        or(flt, feq)
    }
}

#[inline]
fn or_lt_ne(vlt: i64, fne: IFact, vne: i64) -> IFact {
    if vne < vlt {
        Any
    } else {
        fne
    }
}

#[inline]
fn or_lt_gt(flt: IFact, vlt: i64, fgt: IFact, vgt: i64) -> IFact {
    if vgt < vlt {
        Any
    } else {
        or(flt, fgt)
    }
}

#[inline]
fn or_eq_ne(veq: i64, fne: IFact, vne: i64) -> IFact {
    if veq == vne {
        Any
    } else {
        fne
    }
}

#[inline]
fn or_eq_gt(feq: IFact, veq: i64, fgt: IFact, vgt: i64) -> IFact {
    if vgt < veq {
        fgt
    } else if veq == vgt {
        gte(veq)
    } else {
        or(feq, fgt)
    }
}

#[inline]
fn or_ne_gt(fne: IFact, vne: i64, vgt: i64) -> IFact {
    if vne > vgt {
        Any
    } else {
        fne
    }
}

#[inline]
fn and_lt_eq(vlt: i64, veq: i64) -> IFact {
    if veq < vlt {
        Eq(veq)
    } else {
        Empty
    }
}

#[inline]
fn and_lt_ne(flt: IFact, vlt: i64, fne: IFact, vne: i64) -> IFact {
    if vne >= vlt {
        LT(vlt)
    } else {
        and(flt, fne)
    }
}

#[inline]
fn and_lt_gt(vlt: i64, vgt: i64) -> IFact {
    if (vlt - vgt) > 1 {
        Any
    } else {
        Empty
    }
}

#[inline]
fn and_eq_ne(veq: i64, vne: i64) -> IFact {
    if veq != vne {
        Eq(veq)
    } else {
        Empty
    }
}

#[inline]
fn and_eq_gt(veq: i64, vgt: i64) -> IFact {
    if veq > vgt {
        Eq(veq)
    } else {
        Empty
    }
}

#[inline]
fn and_ne_gt(fne: IFact, vne: i64, fgt: IFact, vgt: i64) -> IFact {
    if (vne <= vgt) {
        fgt
    } else {
        and(fne, fgt)
    }
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
                Any => Proved,
                LT(b) => {
                    if a <= b {
                        Proved
                    } else {
                        CannotProve
                    }
                }
                Eq(b) => {
                    if b < a {
                        CannotProve
                    } else {
                        Contradiction(self.clone())
                    }
                }
                NE(_) | GT(_) => Contradiction(self.clone()),
                And(f1, f2) => self.proves_both(f1.borrow(), f2.borrow()),
                Or(f1, f2) => self.proves_any(f1.borrow(), f2.borrow()),
            },
            Eq(a) => match other {
                Empty => Contradiction(self.clone()),
                Any => Proved,
                LT(b) => {
                    if a < b {
                        Proved
                    } else {
                        Contradiction(self.clone())
                    }
                }
                Eq(b) => {
                    if a == b {
                        Proved
                    } else {
                        Contradiction(self.clone())
                    }
                }
                NE(b) => {
                    if a != b {
                        Proved
                    } else {
                        Contradiction(self.clone())
                    }
                }
                GT(b) => {
                    if a > b {
                        Proved
                    } else {
                        Contradiction(self.clone())
                    }
                }
                And(f1, f2) => self.proves_both(f1.borrow(), f2.borrow()),
                Or(f1, f2) => self.proves_any(f1.borrow(), f2.borrow()),
            },
            NE(a) => match other {
                Empty => Contradiction(self.clone()),
                Any => Proved,
                LT(b) => {
                    if a >= b {
                        CannotProve
                    } else {
                        Contradiction(self.clone())
                    }
                }
                Eq(b) => {
                    if a == b {
                        CannotProve
                    } else {
                        Contradiction(self.clone())
                    }
                }
                NE(b) => {
                    if a == b {
                        Proved
                    } else {
                        Contradiction(self.clone())
                    }
                }
                GT(b) => {
                    if a <= b {
                        CannotProve
                    } else {
                        Contradiction(self.clone())
                    }
                }
                And(f1, f2) => self.proves_both(f1.borrow(), f2.borrow()),
                Or(f1, f2) => self.proves_any(f1.borrow(), f2.borrow()),
            },
            GT(a) => match other {
                Empty => Contradiction(self.clone()),
                Any => Proved,
                Eq(b) => {
                    if b > a {
                        CannotProve
                    } else {
                        Contradiction(self.clone())
                    }
                }
                GT(b) => {
                    if a >= b {
                        Proved
                    } else {
                        CannotProve
                    }
                }
                NE(_) | LT(_) => Contradiction(self.clone()),
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
            LT(vlt) => match other {
                Empty => self,
                Any => Any,
                LT(b) => LT(max(vlt, b)),
                Eq(veq) => or_lt_eq(self, vlt, other, veq),
                NE(vne) => or_lt_ne(vlt, other, vne),
                GT(vgt) => or_lt_gt(self, vlt, other, vgt),
                _ => self.proof_based_or(other),
            },
            Eq(veq) => match other {
                Empty => self,
                Any => Any,
                LT(vlt) => or_lt_eq(other, vlt, self, veq),
                Eq(b) => {
                    if veq == b {
                        self
                    } else {
                        or(self, other)
                    }
                }
                NE(vne) => or_eq_ne(veq, other, vne),
                GT(vgt) => or_eq_gt(self, veq, other, vgt),
                _ => self.proof_based_or(other),
            },
            NE(vne) => match other {
                Empty => self,
                Any => Any,
                LT(vlt) => or_lt_ne(vlt, self, vne),
                Eq(veq) => or_eq_ne(veq, self, vne),
                GT(vgt) => or_ne_gt(self, vne, vgt),
                _ => self.proof_based_or(other),
            },
            GT(vgt) => match other {
                Empty => self,
                Any => Any,
                LT(vlt) => or_lt_gt(other, vlt, self, vgt),
                Eq(veq) => or_eq_gt(other, veq, self, vgt),
                NE(vne) => or_ne_gt(other, vne, vgt),
                GT(b) => GT(min(vgt, b)),
                _ => self.proof_based_or(other),
            },
            _ => match other {
                Empty => self,
                Any => Any,
                _ => self.proof_based_or(other),
            },
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
        match self {
            Empty => Empty,
            Any => other,
            LT(vlt) => match other {
                Empty => Empty,
                Any => self,
                LT(vlt2) => LT(min(vlt, vlt2)),
                Eq(veq) => and_lt_eq(vlt, veq),
                NE(vne) => and_lt_ne(self, vlt, other, vne),
                GT(vgt) => and_lt_gt(vlt, vgt),
                _ => self.proof_based_and(other),
            },
            Eq(veq) => match other {
                Empty => Empty,
                Any => self,
                LT(vlt) => and_lt_eq(vlt, veq),
                Eq(veq2) => {
                    if veq == veq2 {
                        self
                    } else {
                        Empty
                    }
                }
                NE(vne) => and_eq_ne(veq, vne),
                GT(vgt) => and_eq_gt(veq, vgt),
                _ => self.proof_based_and(other),
            },
            NE(vne) => match other {
                Empty => Empty,
                Any => self,
                LT(vlt) => and_lt_ne(other, vlt, self, vne),
                Eq(veq) => and_eq_ne(veq, vne),
                NE(vne2) => {
                    if vne == vne2 {
                        self
                    } else {
                        and(self, other)
                    }
                }
                GT(vgt) => and_ne_gt(self, vne, other, vgt),
                _ => self.proof_based_and(other),
            },
            GT(vgt) => match other {
                Empty => Empty,
                Any => self,
                LT(vlt) => and_lt_gt(vlt, vgt),
                Eq(veq) => and_eq_gt(veq, vgt),
                NE(vne) => and_ne_gt(other, vne, self, vgt),
                GT(vgt2) => GT(max(vgt, vgt2)),
                _ => self.proof_based_and(other),
            },
            _ => match other {
                Empty => Empty,
                Any => self,
                _ => self.proof_based_and(other),
            },
        }
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

    #[test]
    fn eq_simple_variant() {
        assert_eq!(Any, Any);
        assert_eq!(LT(2), LT(2));
        assert_eq!(Eq(2), Eq(2));
        assert_eq!(GT(2), GT(2));
    }

    #[test]
    fn ne_simple_variant() {
        assert_ne!(LT(2), LT(4));
        assert_ne!(Eq(2), Eq(4));
        assert_ne!(GT(2), GT(4));
    }

    #[test]
    fn ne_different_variant() {
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
    }

    #[test]
    fn eq_commutative_ops() {
        assert_eq!(and_range(5, 10), reverse(and_range(5, 10)));
        assert_eq!(or_range(5, 10), reverse(or_range(5, 10)));
    }
}
