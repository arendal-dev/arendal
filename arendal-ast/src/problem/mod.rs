use crate::position::Position;
use arcstr::ArcStr;

#[derive(Clone, Debug)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Problem {
    pub severity: Severity,
    pub position: Position,
    pub code: ArcStr,
    pub message: ArcStr,
}

pub trait AResult<T> {
}

pub type Result<T> = std::result::Result<(T, Problems), Problems>;

impl<T> AResult<T> for Result<T> {
}

// Creates an ok result with no warnings
pub fn ok<T>(value: T) -> Result<T> {
    Ok((value, Problems::default()))
}

// Creates a result with a single error
pub fn error<T>(position: Position, code: &str, message: &str) -> Result<T> {
    let error = Problem {
        severity: Severity::Error,
        position,
        code: code.into(),
        message: message.into(),
    };
    Err(Problems { problems: vec![error] })
}

#[derive(Default)]
pub struct Problems {
    problems: Vec<Problem>,
}

impl Problems {
    pub fn add_error(&mut self, position: Position, code: &str, message: String) {
        self.problems.push(Problem {
            severity: Severity::Error,
            position,
            code: code.into(),
            message: message.into(),
        });
    }

    pub fn add_warning(&mut self, position: Position, code: &str, message: &str) {
        self.problems.push(Problem {
            severity: Severity::Warning,
            position,
            code: code.into(),
            message: message.into(),
        });
    }

    pub fn add_problems(&mut self, mut problems: Problems) {
        self.problems.append(&mut problems.problems);
    }

    fn has_error(&self) -> bool {
        self.problems.iter().any(|p| matches!(p.severity, Severity::Error))
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        if self.has_error() {
            Err(self)
        } else {
            Ok((value, self))
        }
    }

    pub fn to_unit_result(self) -> Result<()> {
        self.to_result(())
    }

    pub fn to_lazy_result<T, F>(self, supplier: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        if self.has_error() {
            Err(self)
        } else {
            Ok((supplier(), self))
        }
    }
}
