// Internal fact implementation
#[derive(Debug)]
enum IFact {
    LT(i64),
    LET(i64),
    Eq(i64),
    GET(i64),
    GT(i64),
    And(Box<IFact>, Box<IFact>),
    Or(Box<IFact>, Box<IFact>),
}

use self::IFact::*;
use super::CombinationError;

impl PartialEq for IFact {
    fn eq(&self, other: &IFact) -> bool {
        match self {
            LT(v1) => match other {
                LT(v2) => v1 == v2,
                _ => false,
            },
            LET(v1) => match other {
                LET(v2) => v1 == v2,
                _ => false,
            },
            Eq(v1) => match other {
                Eq(v2) => v1 == v2,
                _ => false,
            },
            GET(v1) => match other {
                GET(v2) => v1 == v2,
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

impl IFact {
    fn or(self, other: IFact) -> IFact {
        match self {
            _ => Or(Box::new(self), Box::new(other)),
        }
    }

    fn and(self, other: IFact) -> Result<IFact, CombinationError> {
        match self {
            _ => Err(CombinationError::Incompatible),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IFact;
    use super::IFact::*;

    fn and(f1: IFact, f2: IFact) -> IFact {
        And(Box::new(f1), Box::new(f2))
    }

    fn and_range(min: i64, max: i64) -> IFact {
        assert!(max > min);
        and(GET(min), LET(max))
    }

    fn or(f1: IFact, f2: IFact) -> IFact {
        Or(Box::new(f1), Box::new(f2))
    }

    fn or_range(min: i64, max: i64) -> IFact {
        assert!(max > min);
        and(LET(min), GET(max))
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
        assert_eq!(LT(2), LT(2));
        assert_ne!(LT(2), LT(4));
        assert_eq!(LET(2), LET(2));
        assert_ne!(LET(2), LET(4));
        assert_eq!(Eq(2), Eq(2));
        assert_ne!(Eq(2), Eq(4));
        assert_eq!(GET(2), GET(2));
        assert_ne!(GET(2), GET(4));
        assert_eq!(GT(2), GT(2));
        assert_ne!(GT(2), GT(4));
        // Different variant
        assert_ne!(LT(2), LET(2));
        assert_ne!(LT(2), Eq(2));
        assert_ne!(LT(2), GET(2));
        assert_ne!(LT(2), GT(2));
        assert_ne!(LT(2), and_range(5, 10));
        assert_ne!(LT(2), or_range(5, 10));
        assert_ne!(LET(2), Eq(2));
        assert_ne!(LET(2), GET(2));
        assert_ne!(LET(2), GT(2));
        assert_ne!(LET(2), and_range(5, 10));
        assert_ne!(LET(2), or_range(5, 10));
        assert_ne!(Eq(2), GET(2));
        assert_ne!(Eq(2), GT(2));
        assert_ne!(Eq(2), and_range(5, 10));
        assert_ne!(Eq(2), or_range(5, 10));
        assert_ne!(GET(2), GT(2));
        assert_ne!(GET(2), and_range(5, 10));
        assert_ne!(GET(2), or_range(5, 10));
        assert_ne!(GT(2), and_range(5, 10));
        assert_ne!(GT(2), or_range(5, 10));
        assert_ne!(and_range(5, 10), or_range(5, 10));
        // Operations are commutative
        assert_eq!(and_range(5, 10), reverse(and_range(5, 10)));
        assert_eq!(or_range(5, 10), reverse(or_range(5, 10)));
    }
}
