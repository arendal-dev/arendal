use std::sync::Arc;

use crate::position::Position;

#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct ProblemInstance {
    position: Position,
    problem: Arc<dyn Problem>,
}

pub trait Problem: std::fmt::Debug {
    fn severity(&self) -> Severity;
}

pub trait AResult<T> {}

pub type Result<T> = std::result::Result<(T, Problems), Problems>;

impl<T> AResult<T> for Result<T> {}

// Creates an ok result with no warnings
pub fn ok<T>(value: T) -> Result<T> {
    Ok((value, Problems::default()))
}

// Creates a result with a single error
pub fn error<T, P: Problem + 'static>(position: Position, problem: P) -> Result<T> {
    let error = ProblemInstance {
        position,
        problem: Arc::new(problem) as Arc<dyn Problem>,
    };
    Err(Problems {
        problems: vec![error],
    })
}

#[derive(Default, Debug)]
pub struct Problems {
    problems: Vec<ProblemInstance>,
}

impl Problems {
    pub fn add<P: Problem + 'static>(&mut self, position: Position, problem: P) {
        self.problems.push(ProblemInstance {
            position,
            problem: Arc::new(problem) as Arc<dyn Problem>,
        });
    }

    pub fn add_problems(&mut self, mut problems: Problems) {
        self.problems.append(&mut problems.problems);
    }

    fn has_error(&self) -> bool {
        self.problems
            .iter()
            .any(|p| matches!(p.problem.severity(), Severity::Error))
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
