use std::sync::Arc;

use crate::position::Position;

pub trait ErrorType: std::fmt::Debug {
    fn at(self, position: Position) -> Error;
}

#[derive(Clone, Debug)]
pub struct Error {
    position: Position,
    error: Arc<dyn ErrorType>,
}

impl Error {
    pub fn new<E: ErrorType + 'static>(position: Position, error: E) -> Self {
        Error {
            position,
            error: Arc::new(error) as Arc<dyn ErrorType>,
        }
    }

    pub fn to_err<T>(self) -> Result<T> {
        Err(Errors {
            errors: vec![self],
            warnings: Vec::new(),
        })
    }
}

#[derive(Debug, Default)]
pub struct Errors {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
}

pub trait WarningType: std::fmt::Debug {
    fn at(self, position: Position) -> Error;
}

#[derive(Clone, Debug)]
pub struct Warning {
    position: Position,
    warning: Arc<dyn WarningType>,
}

impl Warning {
    pub fn new<W: WarningType + 'static>(position: Position, warning: W) -> Self {
        Warning {
            position,
            warning: Arc::new(warning) as Arc<dyn WarningType>,
        }
    }

    pub fn to_ok<T>(self, value: T) -> Result<T> {
        Ok(Warnings {
            warnings: Vec::new(),
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
                    Err(Errors {
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

pub type Result<T> = std::result::Result<Warnings<T>, Errors>;

pub trait ProblemsResult<T> {
    fn and_then_wp<U, F: FnOnce(T) -> Result<U>>(self, op: F) -> Result<U>;
    fn merge<U, V, F: FnOnce(T, U) -> V>(self, other: Result<U>, op: F) -> Result<V>;
}

impl<T> ProblemsResult<T> for Result<T> {
    fn and_then_wp<U, F: FnOnce(T) -> Result<U>>(self, op: F) -> Result<U> {
        match self {
            Ok(w) => w.and_then(op),
            Err(e) => Err(e),
        }
    }

    fn merge<U, V, F: FnOnce(T, U) -> V>(self, other: Result<U>, op: F) -> Result<V> {
        let mut errors: Vec<Error>;
        let mut warnings: Vec<Warning>;
        match self {
            Ok(w1) => {
                warnings = w1.warnings;
                match other {
                    Ok(mut w2) => {
                        warnings.append(&mut w2.warnings);
                        Ok(Warnings {
                            warnings,
                            value: op(w1.value, w2.value),
                        })
                    }
                    Err(mut e2) => {
                        warnings.append(&mut e2.warnings);
                        Err(Errors {
                            errors: e2.errors,
                            warnings,
                        })
                    }
                }
            }
            Err(e1) => {
                errors = e1.errors;
                warnings = e1.warnings;
                match other {
                    Ok(mut w2) => {
                        warnings.append(&mut w2.warnings);
                    }
                    Err(mut e2) => {
                        errors.append(&mut e2.errors);
                        warnings.append(&mut e2.warnings);
                    }
                }
                Err(Errors { errors, warnings })
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Problems {
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Problems {
    // Creates an ok result with no warnings
    pub fn ok<T>(value: T) -> Result<T> {
        Ok(Warnings {
            warnings: Vec::new(),
            value,
        })
    }

    pub fn add_problems(&mut self, mut problems: Problems) {
        self.errors.append(&mut problems.errors);
        self.warnings.append(&mut problems.warnings);
    }

    pub fn add_result<T>(&mut self, mut result: Result<T>) -> Option<T> {
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
        Err(Errors {
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
