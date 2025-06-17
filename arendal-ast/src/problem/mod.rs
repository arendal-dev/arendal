use std::sync::Arc;

use crate::position::Position;

pub trait ErrorType: std::fmt::Debug + 'static {
    fn at(self, position: Position) -> Error
    where
        Self: Sized,
    {
        Error::new(position, self)
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    position: Position,
    error: Arc<dyn ErrorType>,
}

impl Error {
    pub fn new<E: ErrorType>(position: Position, error: E) -> Self {
        Error {
            position,
            error: Arc::new(error) as Arc<dyn ErrorType>,
        }
    }

    pub fn to_err<T>(self) -> Result<T> {
        Err(Problems {
            errors: vec![self],
            warnings: Vec::new(),
        })
    }
}

pub trait WarningType: std::fmt::Debug + 'static {
    fn at(self, position: Position) -> Warning
    where
        Self: Sized,
    {
        Warning::new(position, self)
    }
}

#[derive(Clone, Debug)]
pub struct Warning {
    position: Position,
    warning: Arc<dyn WarningType>,
}

impl Warning {
    pub fn new<W: WarningType>(position: Position, warning: W) -> Self {
        Warning {
            position,
            warning: Arc::new(warning) as Arc<dyn WarningType>,
        }
    }

    pub fn to_ok<T>(self, value: T) -> Result<T> {
        Ok(Warnings {
            warnings: vec![self],
            value,
        })
    }

    pub fn to_warnings(self) -> Warnings<()> {
        Warnings {
            warnings: vec![self],
            value: (),
        }
    }
}

#[derive(Debug)]
pub struct Warnings<T> {
    pub warnings: Vec<Warning>,
    pub value: T,
}

impl<T> Warnings<T> {
    pub fn add(&mut self, warning: Warning) {
        self.warnings.push(warning);
    }

    pub fn to_problems(self) -> (Problems, T) {
        let problems = Problems {
            errors: Vec::default(),
            warnings: self.warnings,
        };
        (problems, self.value)
    }

    pub fn and_then<U, F: FnOnce(T) -> Result<U>>(mut self, op: F) -> Result<U> {
        let result = op(self.value);
        if self.warnings.is_empty() {
            result
        } else {
            match result {
                Ok(mut w2) => {
                    self.warnings.append(&mut w2.warnings);
                    Ok(Warnings {
                        warnings: self.warnings,
                        value: w2.value,
                    })
                }
                Err(mut e2) => {
                    self.warnings.append(&mut e2.warnings);
                    Err(Problems {
                        errors: e2.errors,
                        warnings: self.warnings,
                    })
                }
            }
        }
    }

    pub fn to_result<U>(self, value: U) -> Result<U> {
        Ok(Warnings {
            warnings: self.warnings,
            value,
        })
    }
}

impl<T> Warnings<Option<T>> {
    pub fn and_then_map<U, F: FnOnce(T) -> Result<U>>(self, op: F) -> Result<Option<U>> {
        self.and_then(|oinput| match oinput {
            Some(input) => op(input)?.and_then(|o| ok(Some(o))),
            None => ok(None),
        })
    }
}

pub type Result<T> = std::result::Result<Warnings<T>, Problems>;

#[derive(Default, Debug)]
pub struct Problems {
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Problems {
    pub fn add_problems(&mut self, mut problems: Problems) {
        self.errors.append(&mut problems.errors);
        self.warnings.append(&mut problems.warnings);
    }

    pub fn add_result<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(mut w) => {
                self.warnings.append(&mut w.warnings);
                Some(w.value)
            }
            Err(mut e) => {
                self.errors.append(&mut e.errors);
                self.warnings.append(&mut e.warnings);
                None
            }
        }
    }

    fn to_ok<T>(self, value: T) -> Result<T> {
        Ok(Warnings {
            warnings: Vec::default(),
            value,
        })
    }

    fn to_err<T>(self) -> Result<T> {
        Err(Problems {
            errors: self.errors,
            warnings: self.warnings,
        })
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            self.to_ok(value)
        } else {
            self.to_err()
        }
    }

    pub fn to_unit_result(self) -> Result<()> {
        self.to_result(())
    }

    pub fn to_lazy_result<T, F>(self, supplier: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        if self.errors.is_empty() {
            self.to_ok(supplier())
        } else {
            self.to_err()
        }
    }
}

impl From<Error> for Problems {
    fn from(value: Error) -> Self {
        Problems {
            errors: vec![value],
            warnings: Vec::new(),
        }
    }
}

// Creates an ok result with no warnings
pub fn ok<T>(value: T) -> Result<T> {
    Ok(Warnings {
        warnings: Vec::new(),
        value,
    })
}

pub fn merge<U, V>(one: Result<U>, two: Result<V>) -> Result<(U, V)> {
    let mut errors: Vec<Error>;
    let mut warnings: Vec<Warning>;
    match one {
        Ok(w1) => {
            warnings = w1.warnings;
            match two {
                Ok(mut w2) => {
                    warnings.append(&mut w2.warnings);
                    Ok(Warnings {
                        warnings,
                        value: (w1.value, w2.value),
                    })
                }
                Err(mut e2) => {
                    warnings.append(&mut e2.warnings);
                    Err(Problems {
                        errors: e2.errors,
                        warnings,
                    })
                }
            }
        }
        Err(e1) => {
            errors = e1.errors;
            warnings = e1.warnings;
            match two {
                Ok(mut w2) => {
                    warnings.append(&mut w2.warnings);
                }
                Err(mut e2) => {
                    errors.append(&mut e2.errors);
                    warnings.append(&mut e2.warnings);
                }
            }
            Err(Problems { errors, warnings })
        }
    }
}

pub struct Output<T> {
    value: Option<T>,
    problems: Problems,
}

impl<T> Output<T> {
    pub fn new() -> Output<T> {
        Output {
            value: None,
            problems: Problems::default(),
        }
    }

    pub fn ok(value: T) -> Output<T> {
        Output {
            value: Some(value),
            problems: Problems::default(),
        }
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.value.replace(value)
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Output<U> {
        Output {
            value: self.value.map(op),
            problems: self.problems,
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> Output<U>>(mut self, op: F) -> Output<U> {
        match self.value {
            None => Output {
                value: None,
                problems: self.problems,
            },
            Some(t) => {
                let other = op(t);
                self.problems.add_problems(other.problems);
                Output {
                    value: other.value,
                    problems: self.problems,
                }
            }
        }
    }

    pub fn merge<U, V, F: FnOnce(T, U) -> V>(mut self, other: Output<U>, op: F) -> Output<V> {
        self.problems.add_problems(other.problems);
        let value = match self.value {
            Some(t) => match other.value {
                Some(u) => Some(op(t, u)),
                _ => None,
            },
            _ => None,
        };
        Output {
            value,
            problems: self.problems,
        }
    }

    pub fn merge_to_tuple<U>(self, other: Output<U>) -> Output<(T, U)> {
        self.merge(other, |t, u| (t, u))
    }

    pub fn merge_problems<U>(&mut self, other: Output<U>) -> Option<U> {
        self.problems.add_problems(other.problems);
        other.value
    }

    pub fn to_result(self) -> Result<T> {
        if self.value.is_some() && self.problems.errors.is_empty() {
            Ok(Warnings {
                warnings: self.problems.warnings,
                value: self.value.unwrap(),
            })
        } else {
            Err(self.problems)
        }
    }

    pub fn add_error(&mut self, error: Error) {
        self.problems.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: Warning) {
        self.problems.warnings.push(warning);
    }
}

impl<T> Output<Vec<T>> {
    pub fn empty() -> Self {
        Output {
            value: Some(Vec::new()),
            problems: Problems::default(),
        }
    }

    pub fn add_value(&mut self, value: T) {
        self.value.as_mut().unwrap().push(value);
    }

    pub fn add_output(&mut self, output: Output<T>) {
        self.problems.add_problems(output.problems);
        output.value.map(|t| self.add_value(t));
    }
}

impl<T> From<Error> for Output<T> {
    fn from(value: Error) -> Output<T> {
        Output {
            value: None,
            problems: value.into(),
        }
    }
}
