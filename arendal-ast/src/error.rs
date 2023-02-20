pub trait Error: std::fmt::Debug {}

type ErrorItem = Box<dyn Error>;
type ErrorVec = Vec<ErrorItem>;

#[derive(Debug, Default)]
pub struct Errors {
    errors: ErrorVec,
}

pub type Result<T> = std::result::Result<T, Errors>;

impl Errors {
    pub fn add<T: Error + 'static>(&mut self, error: T) {
        self.errors.push(Box::new(error));
    }

    pub fn add_option<T: Error + 'static>(&mut self, error: Option<T>) {
        if let Some(e) = error {
            self.add(e)
        }
    }

    pub fn append(&mut self, mut other: Errors) {
        self.errors.append(&mut other.errors)
    }

    pub fn append_result<T>(&mut self, result: Result<T>) {
        if let Err(other) = result {
            self.append(other);
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }

    pub fn result_to_result<T>(mut self, result: Result<T>) -> Result<T> {
        match result {
            Ok(value) => self.to_result(value),
            Err(errors) => {
                self.append(errors);
                Err(self)
            }
        }
    }
}
