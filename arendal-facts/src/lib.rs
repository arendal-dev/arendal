type Subject = u64;

#[derive(Debug)]
enum Fact {
    UnaryBool(Subject, bool),
}

use crate::Fact::*;

#[derive(PartialEq, Debug)]
enum AddsKnowledge {
    Yes,
    No,
    Supersedes,
    Incompatible,
}

use crate::AddsKnowledge::*;

impl Fact {
    fn add_knowledge(&self, other: &Fact) -> AddsKnowledge {
        match *self {
            UnaryBool(s1, b1) => match *other {
                UnaryBool(s2, b2) => {
                    if s1 != s2 {
                        Yes
                    } else if b1 == b2 {
                        No
                    } else {
                        Incompatible
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AddsKnowledge::*;
    use crate::Fact::*;

    #[test]
    fn unary_boolean_facts() {
        assert_eq!(UnaryBool(1, true).add_knowledge(&UnaryBool(2, true)), Yes);
        assert_eq!(UnaryBool(1, true).add_knowledge(&UnaryBool(1, true)), No);
        assert_eq!(
            UnaryBool(1, true).add_knowledge(&UnaryBool(1, false)),
            Incompatible
        );
    }
}
