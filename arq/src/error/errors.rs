use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::ErrorKind;

pub struct GenericError {
    message: String,
    source: Option<Box<GenericError>>
}

impl Debug for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let main_error = write!(f, "Error: {}", self.message);
        if main_error.is_ok() {
            if self.source.is_some() {
                let caused_by = write!(f, "Caused by:");
                if caused_by.is_ok() {
                    return self.source.fmt(f);
                } else {
                    return caused_by;
                }
            }
        }
        main_error

    }
}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for GenericError {
    
}

impl GenericError {
    pub fn new(message: String) -> GenericError {
        GenericError { message, source: None }
    }

    pub fn new_with_source(source: GenericError, message: String) -> GenericError {
        GenericError { message, source: Some(Box::new(source)) }
    }

    pub fn to_io_error(self) -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, format!("{}", self))
    }
}