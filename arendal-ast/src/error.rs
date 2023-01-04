pub trait Error: std::fmt::Debug {}

type ErrorItem = Box<dyn Error>;
type ErrorVec = Vec<ErrorItem>;

#[derive(Debug)]
pub struct Errors {
    errors: ErrorVec,
}

pub type Result<T> = std::result::Result<T, Errors>;

impl Errors {
    pub fn new() -> Errors {
        Errors { errors: Vec::new() }
    }

    pub fn add<T: Error + 'static>(&mut self, error: T) {
        self.errors.push(Box::new(error));
    }

    pub fn add_option<T: Error + 'static>(&mut self, error: Option<T>) {
        match error {
            Some(e) => self.add(e),
            None => (),
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}
