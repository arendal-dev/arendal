mod ubool;
mod uint;

type Subject = u64;

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

#[derive(Debug)]
enum Fact {
    UnaryBool(Subject, ubool::Fact),
}

use self::Fact::*;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::Fact;
    use super::Fact::*;
    use super::Subject;

    #[test]
    fn test() {}
}
