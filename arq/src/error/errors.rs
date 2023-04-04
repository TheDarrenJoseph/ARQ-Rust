use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct GenericError {
    message: String
}

impl Debug for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
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
        GenericError { message }
    }
}