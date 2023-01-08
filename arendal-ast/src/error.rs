pub trait ErrorLoc {}
pub trait Error<'a, L: ErrorLoc>: std::fmt::Debug {
    fn location(&self) -> L;
}

type ErrorItem<'a, L> = Box<dyn Error<'a, L> + 'a>;
type ErrorVec<'a, L> = Vec<ErrorItem<'a, L>>;

#[derive(Debug)]
pub struct Errors<'a, L> {
    errors: ErrorVec<'a, L>,
}

pub type Result<'a, T, L> = std::result::Result<T, Errors<'a, L>>;

impl<'a, L: ErrorLoc> Errors<'a, L> {
    pub fn new() -> Errors<'a, L> {
        Errors { errors: Vec::new() }
    }

    pub fn add<T: Error<'a, L> + 'a>(&mut self, error: T) {
        self.errors.push(Box::new(error));
    }

    pub fn add_option<T: Error<'a, L> + 'a>(&mut self, error: Option<T>) {
        match error {
            Some(e) => self.add(e),
            None => (),
        }
    }

    pub fn to_result<T>(self, value: T) -> Result<'a, T, L> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}
