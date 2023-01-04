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

pub fn merge(e1 : Error, e2 : Error) -> Error {
    match e1 {
        Single(i1) => match e2 {
            Single(i2) => Multiple(vec![i1, i2]),
            Multiple(mut v2) => {
                v2.insert(0, i1);
                Multiple(v2)
            }
        },
        Multiple(mut v1) => match e2 {
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

pub fn merge_options(o1 : Option<Error>, o2 : Option<Error>) -> Option<Error> {
    match o1 {
        Some(e1) => match o2 {
            Some(e2) => Some(merge(e1, e2)),
            None => Some(e1)
        },
        None => o2,
    }
}


pub type Result<T> = std::result::Result<T, Error>;

pub mod errors {
    pub trait Error : std::fmt::Debug {

    }
}
