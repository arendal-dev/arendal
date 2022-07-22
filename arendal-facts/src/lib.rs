mod ubool;
mod uint;

type Subject = u64;

use std::cmp::Ordering;
use std::rc::Rc;

#[derive(PartialEq, Debug)]
enum Proof<T> {
    Proved,
    CannotProve,
    Contradiction(T),
}

use self::Proof::*;

impl<T: Eq> Proof<T> {
    fn is_proved(&self) -> bool {
        *self == Proved
    }

    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Proof<U> {
        match self {
            Proved => Proved,
            CannotProve => CannotProve,
            Contradiction(t) => Contradiction(f(t)),
        }
    }
}

#[derive(PartialEq, Debug)]
enum CombinationError {
    Incompatible,
}

#[derive(PartialEq, Debug)]
enum UnaryFact {
    UnaryBool(bool),
    UnaryInt(Ordering, u64),
    UnaryOr(Rc<UnaryFact>, Rc<UnaryFact>),
}

use crate::UnaryFact::*;

#[derive(Debug)]
enum Fact {
    Unary(Subject, Rc<UnaryFact>),
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

    fn or(f1: Rc<UnaryFact>, f2: Rc<UnaryFact>) -> Result<Rc<UnaryFact>, Assessment> {
        match f1.assess(&f2) {
            Equal | SelfImpliesOther => Ok(f1),
            OtherImpliesSelf => Ok(f2),
            Compatible | Incompatible => Ok(Rc::new(UnaryOr(f1, f2))),
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
    use std::rc::Rc;

    use crate::Assessment;
    use crate::Assessment::*;
    use crate::Fact;
    use crate::Fact::*;
    use crate::Subject;
    use crate::UnaryFact;
    use crate::UnaryFact::*;

    fn rcf(value: bool) -> Rc<UnaryFact> {
        Rc::new(UnaryBool(value))
    }

    fn unary_bool(s: Subject, value: bool) -> Fact {
        Unary(s, rcf(value))
    }

    #[test]
    fn test_unary_fact_bool() {
        fn ubt() -> UnaryFact {
            UnaryBool(true)
        }

        fn ubf() -> UnaryFact {
            UnaryBool(false)
        }

        assert_eq!(ubt().assess(&ubt()), Equal);
        assert_eq!(ubf().assess(&ubf()), Equal);
        assert_eq!(ubt().assess(&ubf()), Incompatible);
        assert_eq!(ubf().assess(&ubt()), Incompatible);

        fn or(f1: bool, f2: bool) -> Result<Rc<UnaryFact>, Assessment> {
            UnaryFact::or(rcf(f1), rcf(f2))
        }

        fn or_result(f1: bool, f2: bool) -> Result<Rc<UnaryFact>, Assessment> {
            Ok(Rc::new(UnaryOr(rcf(f1), rcf(f2))))
        }

        assert_eq!(or(true, true), Ok(rcf(true)));
        assert_eq!(or(false, false), Ok(rcf(false)));
        assert_eq!(or(true, false), or_result(true, false));
        assert_eq!(or(false, true), or_result(false, true));
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
