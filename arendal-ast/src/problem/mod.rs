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

pub type Problems = Vec<Problem>;

pub struct Output<T> {
    pub value: T,
    pub warnings: Vec<Problem>,
}

pub trait AResult<T> {
    fn builder() -> ResultBuilder;
}

pub type Result<T> = std::result::Result<Output<T>, Problems>;

impl<T> AResult<T> for Result<T> {
    fn builder() -> ResultBuilder {
        ResultBuilder {
            problems: Vec::default(),
            has_error: false,
        }
    }
}

// Creates an ok result with no warnings
pub fn ok<T>(value: T) -> Result<T> {
    Ok(Output {
        value,
        warnings: Vec::default(),
    })
}

// Creates a result with a single error
pub fn error<T>(position: Position, code: &str, message: &str) -> Result<T> {
    let error = Problem {
        severity: Severity::Error,
        position,
        code: code.into(),
        message: message.into(),
    };
    Err(vec![error])
}

pub struct ResultBuilder {
    problems: Problems,
    has_error: bool,
}

impl ResultBuilder {
    pub fn add_error(&mut self, position: Position, code: &str, message: &str) {
        self.problems.push(Problem {
            severity: Severity::Error,
            position,
            code: code.into(),
            message: message.into(),
        });
        self.has_error = true;
    }

    pub fn add_warning(&mut self, position: Position, code: &str, message: &str) {
        self.problems.push(Problem {
            severity: Severity::Warning,
            position,
            code: code.into(),
            message: message.into(),
        });
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        if self.has_error {
            Err(self.problems)
        } else {
            Ok(Output {
                value,
                warnings: self.problems,
            })
        }
    }

    pub fn to_unit_result(self) -> Result<()> {
        self.to_result(())
    }

    pub fn to_lazy_result<T, F>(self, supplier: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        if self.has_error {
            Err(self.problems)
        } else {
            Ok(Output {
                value: supplier(),
                warnings: self.problems,
            })
        }
    }
}
