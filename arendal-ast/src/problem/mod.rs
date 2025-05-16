use std::sync::Arc;

use crate::position::Position;

pub trait ErrorType: std::fmt::Debug {}

#[derive(Clone, Debug)]
pub struct Error {
    position: Position,
    error: Arc<dyn ErrorType>,
}

#[derive(Debug, Default)]
pub struct Errors {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
}

pub trait WarningType: std::fmt::Debug {}

#[derive(Clone, Debug)]
pub struct Warning {
    position: Position,
    warning: Arc<dyn WarningType>,
}

#[derive(Debug)]
pub struct Warnings<T> {
    pub warnings: Vec<Warning>,
    pub value: T,
}

impl<T> Warnings<T> {
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
                        ok_w(warnings, op(w1.value, w2.value))
                    }
                    Err(mut e2) => {
                        warnings.append(&mut e2.warnings);
                        err(e2.errors, warnings)
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
                err(errors, warnings)
            }
        }
    }
}

// Creates an ok result with no warnings
pub fn ok<T>(value: T) -> Result<T> {
    Ok(Warnings {
        warnings: Vec::default(),
        value,
    })
}

// Creates a result with a single error
pub fn error<T, E: ErrorType + 'static>(position: Position, error: E) -> Result<T> {
    let error = Error {
        position,
        error: Arc::new(error) as Arc<dyn ErrorType>,
    };
    Err(Errors {
        errors: vec![error],
        warnings: Vec::default(),
    })
}

fn err<T>(errors: Vec<Error>, warnings: Vec<Warning>) -> Result<T> {
    Err(Errors { errors, warnings })
}

fn ok_w<T>(warnings: Vec<Warning>, value: T) -> Result<T> {
    Ok(Warnings { warnings, value })
}

#[derive(Default, Debug)]
pub struct Problems {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
}

impl Problems {
    pub fn add_error<E: ErrorType + 'static>(&mut self, position: Position, error: E) {
        self.errors.push(Error {
            position,
            error: Arc::new(error) as Arc<dyn ErrorType>,
        });
    }

    pub fn add_warning<W: WarningType + 'static>(&mut self, position: Position, error: W) {
        self.warnings.push(Warning {
            position,
            warning: Arc::new(error) as Arc<dyn WarningType>,
        });
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
