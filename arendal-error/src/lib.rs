type ErrorItem = Box<dyn errors::Error>;
type ErrorVec = Vec<Box<dyn errors::Error>>;

#[derive(Debug)]
pub enum Error {
    Single(ErrorItem),
    Multiple(ErrorVec),
}

use Error::{Single, Multiple};

#[inline]
pub fn error<T : errors::Error + 'static>(err : T) -> Error {
    Single(Box::new(err))
}

impl Error {
    pub fn merge(self, error : Error) -> Error {
        match self {
            Single(i1) => match error {
                Single(i2) => Multiple(vec![i1, i2]),
                Multiple(mut v2) => {
                    v2.insert(0, i1);
                    Multiple(v2)
                }
            },
            Multiple(mut v1) => match error {
                Single(i2) => {
                    v1.push(i2);
                    Multiple(v1)
                }
                Multiple(mut v2) => {
                    v1.append(&mut v2);
                    Multiple(v1)
                }
            }
        }
    }
}

pub struct ErrorCollector {
    error : Option<Error>
}

impl ErrorCollector {
    pub fn new() -> ErrorCollector {
        ErrorCollector { error: None }
    }

    pub fn add(&mut self,  error : Error) {
        match &mut self.error {
            None => self.error = Some(error),
            _ => {
                let e = self.error.take().unwrap();
                self.error = Some(e.merge(error));
            },
        }
    }

    pub fn add_option(&mut self,  error : Option<Error>) {
        match error {
            Some(e) => self.add(e),
            None => (),
        }
    }

    pub fn to_result<T>(self, value : T) -> Result<T> {
        match self.error {
            None => Ok(value),
            Some(e) => Err(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod errors {
    pub trait Error : std::fmt::Debug {

    }
}
