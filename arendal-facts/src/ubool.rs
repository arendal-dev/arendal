/// Internal representation of a fact
#[derive(PartialEq, Eq, Debug)]
enum IFact {
    True,
    False,
    Any,
}

use self::IFact::*;
use super::Proof;
use super::Proof::*;

impl IFact {
    fn or(self, other: IFact) -> IFact {
        if self == other {
            self
        } else {
            Any
        }
    }

    fn and(self, other: IFact) -> Result<IFact, IFact> {
        match self {
            True => match other {
                True => Ok(True),
                False => Err(True),
                Any => Ok(True),
            },
            False => match other {
                True => Err(False),
                False => Ok(False),
                Any => Ok(False),
            },
            Any => Ok(other),
        }
    }

    fn proves(self, other: IFact) -> Proof<IFact> {
        match self {
            True => match other {
                True | Any => Proved,
                False => Contradiction(True),
            },
            False => match other {
                False | Any => Proved,
                True => Contradiction(False),
            },
            Any => match other {
                True | False => CannotProve,
                Any => Proved,
            },
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

    fn and(self, other: Fact) -> Result<Fact, Fact> {
        self.fact
            .and(other.fact)
            .map(Self::create)
            .map_err(Self::create)
    }

    fn proves(self, other: Fact) -> Proof<Fact> {
        self.fact.proves(other.fact).map(Self::create)
    }
}

#[cfg(test)]
mod tests {
    use super::IFact::*;
    use super::Proof::*;

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
        assert_eq!(True.and(True), Ok(True));
        assert_eq!(False.and(False), Ok(False));
        assert_eq!(True.and(False), Err(True));
        assert_eq!(False.and(True), Err(False));
        assert_eq!(True.and(Any), Ok(True));
        assert_eq!(False.and(Any), Ok(False));
        assert_eq!(Any.and(True), Ok(True));
        assert_eq!(Any.and(False), Ok(False));
    }

    #[test]
    fn proofs() {
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
        assert_eq!(t().and(f()), Err(t()));
    }

    #[test]
    fn external_proof() {
        assert_eq!(t().proves(t()), Proved);
        assert_eq!(t().proves(f()), Contradiction(t()));
    }
}
