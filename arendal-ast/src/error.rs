pub trait Error<'a>: std::fmt::Debug {}

type ErrorItem<'a> = Box<dyn Error<'a> + 'a>;
type ErrorVec<'a> = Vec<ErrorItem<'a>>;

#[derive(Debug, Default)]
pub struct Errors<'a> {
    errors: ErrorVec<'a>,
}

pub type Result<'a, T> = std::result::Result<T, Errors<'a>>;

impl<'a> Errors<'a> {
    pub fn add<T: Error<'a> + 'a>(&mut self, error: T) {
        self.errors.push(Box::new(error));
    }

    pub fn add_option<T: Error<'a> + 'a>(&mut self, error: Option<T>) {
        if let Some(e) = error {
            self.add(e)
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
