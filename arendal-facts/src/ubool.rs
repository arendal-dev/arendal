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
}

/// Public representation of a fact
#[derive(PartialEq, Eq, Debug)]
pub struct Fact {
    fact: IFact,
}

impl Fact {
    fn new(value: bool) -> Fact {
        Fact {
            fact: if value { True } else { False },
        }
    }

    fn or(self, other: Fact) -> Fact {
        Fact {
            fact: self.fact.or(other.fact),
        }
    }

    fn and(self, other: Fact) -> Option<Fact> {
        self.fact.and(other.fact).map(|f| Fact { fact: f })
    }
}

#[cfg(test)]
mod tests {
    use super::IFact;
    use super::IFact::*;

    #[test]
    fn test_internal_or() {
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
    fn test_internal_and() {
        assert_eq!(True.and(True), Some(True));
        assert_eq!(False.and(False), Some(False));
        assert_eq!(True.and(False), None);
        assert_eq!(False.and(True), None);
        assert_eq!(True.and(Any), Some(True));
        assert_eq!(False.and(Any), Some(False));
        assert_eq!(Any.and(True), Some(True));
        assert_eq!(Any.and(False), Some(False));
    }

    use super::Fact;

    #[test]
    fn test_new() {
        assert_eq!(Fact::new(true).fact, True);
        assert_eq!(Fact::new(false).fact, False);
    }

    #[test]
    fn test_external_operator() {
        assert_eq!(Fact::new(true).or(Fact::new(false)).fact, Any);
        assert_eq!(Fact::new(true).and(Fact::new(false)), None);
    }
}
