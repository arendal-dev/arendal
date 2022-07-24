mod ubool;
mod uint;

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
enum Fact<S: PartialEq + std::fmt::Debug> {
    UnaryBool(S, ubool::Fact),
}

use self::Fact::*;

impl<S: PartialEq + std::fmt::Debug> Fact<S> {}

#[cfg(test)]
mod tests {

    type Fact = super::Fact<usize>;
    use super::ubool::Fact as BoolFact;
    use super::Fact::*;

    #[test]
    fn eq() {
        assert_eq!(UnaryBool(1, BoolFact::True), UnaryBool(1, BoolFact::True));
    }
}
