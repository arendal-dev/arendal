use super::ArcStr;
use std::fmt;

pub trait Position: Clone {}

#[derive(Clone)]
pub struct NoPosition {}

impl Position for NoPosition {}

impl NoPosition {
    #[inline]
    pub fn get() -> NoPosition {
        NoPosition {}
    }
}

pub trait ProblemCode: Clone + PartialEq + Eq + fmt::Display {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorCode {
    code: ArcStr,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.code.fmt(f)
    }
}

impl ProblemCode for ErrorCode {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarningCode {
    code: ArcStr,
}

impl fmt::Display for WarningCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.code.fmt(f)
    }
}

impl ProblemCode for WarningCode {}

pub trait Problem<P: Position, C: ProblemCode> {
    fn position(&self) -> P;
    fn code(&self) -> C;
    fn message(&self) -> ArcStr;
}

#[derive(Clone, Debug)]
pub struct Error<P: Position> {
    pub position: P,
    pub code: ErrorCode,
    pub message: ArcStr,
}

impl<P: Position> Problem<P, ErrorCode> for Error<P> {
    fn position(&self) -> P {
        self.position.clone()
    }

    fn code(&self) -> ErrorCode {
        self.code.clone()
    }

    fn message(&self) -> ArcStr {
        self.message.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Warning<P: Position> {
    pub position: P,
    pub code: WarningCode,
    pub message: ArcStr,
}

impl<P: Position> Problem<P, WarningCode> for Warning<P> {
    fn position(&self) -> P {
        self.position.clone()
    }

    fn code(&self) -> WarningCode {
        self.code.clone()
    }

    fn message(&self) -> ArcStr {
        self.message.clone()
    }
}

pub type Errors<P> = Vec<Error<P>>;

pub type Warnings<P> = Vec<Warning<P>>;

pub struct Problems<P: Position> {
    pub errors: Errors<P>,
    pub warnings: Warnings<P>,
}

pub struct Output<P: Position, T> {
    pub value: T,
    pub warnings: Warnings<P>,
}

impl<P: Position, T> Output<P, T> {
    pub fn builder() -> ResultBuilder<P> {
        ResultBuilder {
            problems: Problems {
                errors: Vec::default(),
                warnings: Vec::default(),
            },
        }
    }
}

pub type Result<P, T> = std::result::Result<Output<P, T>, Problems<P>>;

pub struct ResultBuilder<P: Position> {
    problems: Problems<P>,
}

impl<P: Position> ResultBuilder<P> {
    pub fn add_error(&mut self, position: P, code: ErrorCode, message: ArcStr) {
        self.problems.errors.push(Error {
            position,
            code,
            message,
        });
    }

    pub fn add_warning(&mut self, position: P, code: WarningCode, message: ArcStr) {
        self.problems.warnings.push(Warning {
            position,
            code,
            message,
        });
    }

    pub fn to_result<T>(self, value: T) -> Result<P, T> {
        if self.problems.errors.is_empty() {
            Ok(Output {
                value,
                warnings: self.problems.warnings,
            })
        } else {
            Err(self.problems)
        }
    }

    pub fn to_unit_result(self) -> Result<P, ()> {
        self.to_result(())
    }

    pub fn to_lazy_result<T, F>(self, supplier: F) -> Result<P, T>
    where
        F: FnOnce() -> T,
    {
        if self.problems.errors.is_empty() {
            Ok(Output {
                value: supplier(),
                warnings: self.problems.warnings,
            })
        } else {
            Err(self.problems)
        }
    }
}
