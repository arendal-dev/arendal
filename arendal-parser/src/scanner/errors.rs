use arendal_error::Error as E;
use arendal_error::error;
use arendal_error::errors::Error;

#[derive(Debug)]
struct IndentationError;

impl Error for IndentationError {
}

pub fn indentation_error() -> E {
    error(IndentationError {})
}
