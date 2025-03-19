use super::ArcStr;
use std::fmt;

pub trait Position: Clone {}

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

pub struct Output<P: Position, T> {
    value: T,
    warnings: Warnings<P>,
}

pub type Result<P, T> = std::result::Result<Output<P, T>, Errors<P>>;
