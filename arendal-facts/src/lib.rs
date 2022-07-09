type Subject = u64;

use std::cmp::Ordering;

#[derive(PartialEq, Debug)]
enum UnaryFact {
    UnaryBool(bool),
    UnaryInt(Ordering, u64),
    UnaryOr(Box<UnaryFact>, Box<UnaryFact>),
}

use crate::UnaryFact::*;

#[derive(Debug)]
enum Fact {
    Unary(Subject, UnaryFact),
}

use crate::Fact::*;

#[derive(PartialEq, Debug)]
enum Assessment {
    Equal,
    SelfImpliesOther,
    OtherImpliesSelf,
    Compatible,
    Incompatible,
}

use crate::Assessment::*;

impl UnaryFact {
    fn assess(&self, other: &UnaryFact) -> Assessment {
        match self {
            UnaryBool(b1) => match other {
                UnaryBool(b2) => {
                    if b1 == b2 {
                        Equal
                    } else {
                        Incompatible
                    }
                }
                _ => Incompatible,
            },
            _ => Incompatible,
        }
    }

    fn or(self, f2: UnaryFact) -> Result<UnaryFact, Assessment> {
        match self.assess(&f2) {
            Equal | SelfImpliesOther => Ok(self),
            OtherImpliesSelf => Ok(f2),
            Compatible => Ok(UnaryOr(Box::new(self), Box::new(f2))),
            _ => Err(Incompatible),
        }
    }
}

impl Fact {
    fn assess(&self, other: &Fact) -> Assessment {
        match self {
            Unary(s1, f1) => match other {
                Unary(s2, f2) => {
                    if s1 != s2 {
                        Compatible
                    } else {
                        f1.assess(&f2)
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Assessment::*;
    use crate::Fact;
    use crate::Fact::*;
    use crate::Subject;
    use crate::UnaryFact::*;

    fn unary_bool(s: Subject, value: bool) -> Fact {
        Unary(s, UnaryBool(value))
    }

    #[test]
    fn test_unary_bool() {
        let t = UnaryBool(true);
        let f = UnaryBool(false);
        assert_eq!(UnaryBool(true).assess(&UnaryBool(true)), Equal);
        assert_eq!(f.assess(&f), Equal);
        assert_eq!(UnaryBool(true).assess(&f), Incompatible);
        assert_eq!(f.assess(&UnaryBool(true)), Incompatible);
        assert_eq!(UnaryBool(true).or(UnaryBool(true)), Ok(UnaryBool(true)));
        assert_eq!(UnaryBool(false).or(UnaryBool(false)), Ok(UnaryBool(false)));
        assert_eq!(UnaryBool(true).or(UnaryBool(false)), Err(Incompatible));
        assert_eq!(UnaryBool(false).or(UnaryBool(true)), Err(Incompatible));
    }

    #[test]
    fn unary_boolean_facts() {
        let s1t = unary_bool(1, true);
        let s2t = unary_bool(2, true);
        let s1f = unary_bool(1, false);
        assert_eq!(s1t.assess(&s2t), Compatible);
        assert_eq!(s1t.assess(&s1t), Equal);
        assert_eq!(s1t.assess(&s1f), Incompatible);
    }
}
