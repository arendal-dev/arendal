/// Internal representation of a fact
#[derive(PartialEq, Eq, Debug)]
enum IFact {
    True,
    False,
    Any,
}

use self::IFact::*;

impl IFact {
    fn or(self, other: IFact) -> IFact {
        if self == other {
            self
        } else {
            Any
        }
    }

    fn and(self, other: IFact) -> Option<IFact> {
        if self == other {
            Some(self)
        } else if self == Any {
            Some(other)
        } else if other == Any {
            Some(self)
        } else {
            None
        }
    }

    fn proves(self, other: IFact) -> bool {
        match self {
            True => other != False,
            False => other != True,
            Any => other == Any,
        }
    }
}

/// Public representation of a fact
#[derive(PartialEq, Eq, Debug)]
pub struct Fact {
    fact: IFact,
}

impl Fact {
    #[inline]
    fn create(fact: IFact) -> Fact {
        Fact { fact: fact }
    }

    fn new(value: bool) -> Fact {
        Self::create(if value { True } else { False })
    }

    fn or(self, other: Fact) -> Fact {
        Self::create(self.fact.or(other.fact))
    }

    fn and(self, other: Fact) -> Option<Fact> {
        self.fact.and(other.fact).map(Self::create)
    }

    fn proves(self, other: Fact) -> bool {
        self.fact.proves(other.fact)
    }
}

#[cfg(test)]
mod tests {
    use super::IFact::*;

    #[test]
    fn internal_or() {
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
    fn internal_and() {
        assert_eq!(True.and(True), Some(True));
        assert_eq!(False.and(False), Some(False));
        assert_eq!(True.and(False), None);
        assert_eq!(False.and(True), None);
        assert_eq!(True.and(Any), Some(True));
        assert_eq!(False.and(Any), Some(False));
        assert_eq!(Any.and(True), Some(True));
        assert_eq!(Any.and(False), Some(False));
    }

    #[test]
    fn proofs() {
        assert!(True.proves(True));
        assert!(!True.proves(False));
        assert!(True.proves(Any));
        assert!(False.proves(False));
        assert!(!False.proves(True));
        assert!(False.proves(Any));
        assert!(Any.proves(Any));
        assert!(!Any.proves(True));
        assert!(!Any.proves(False));
    }

    use super::Fact;

    fn t() -> Fact {
        Fact::new(true)
    }

    fn f() -> Fact {
        Fact::new(false)
    }

    #[test]
    fn new() {
        assert_eq!(t().fact, True);
        assert_eq!(f().fact, False);
    }

    #[test]
    fn external_operator() {
        assert_eq!(t().or(f()).fact, Any);
        assert_eq!(t().and(f()), None);
    }

    #[test]
    fn external_proof() {
        assert!(t().proves(t()));
        assert!(!t().proves(f()));
    }
}
