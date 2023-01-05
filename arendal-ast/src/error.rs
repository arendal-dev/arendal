pub trait Error: std::fmt::Debug {}

type ErrorItem<'a> = Box<dyn Error + 'a>;
type ErrorVec<'a> = Vec<ErrorItem<'a>>;

#[derive(Debug)]
pub struct Errors<'a> {
    errors: ErrorVec<'a>,
}

pub type Result<'a, T> = std::result::Result<T, Errors<'a>>;

impl<'a> Errors<'a> {
    pub fn new() -> Errors<'a> {
        Errors { errors: Vec::new() }
    }

    pub fn add<T: Error + 'a>(&mut self, error: T) {
        self.errors.push(Box::new(error));
    }

    pub fn add_option<T: Error + 'static>(&mut self, error: Option<T>) {
        match error {
            Some(e) => self.add(e),
            None => (),
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<'a, T> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}
