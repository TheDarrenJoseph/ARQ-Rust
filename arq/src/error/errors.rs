use std::error::Error;
use std::fmt::{Debug, Display, Formatter, write};
use std::io;
use std::io::ErrorKind;
use crate::error::errors::ErrorType::{INTERNAL, IO};

pub enum ErrorType {
    INTERNAL,
    IO
}

// TODO hookup for messages
pub struct ErrorWrapper {
    pub(crate) message: Option<String>, // For non-critical / system errors
    pub(crate) io_error: Option<std::io::Error>,
    pub(crate) error_type: ErrorType
}

impl ErrorWrapper {
    pub(crate) const fn new_internal(message: String) -> ErrorWrapper {
        ErrorWrapper { message: Some(message), io_error: None,  error_type: INTERNAL }
    }

    pub(crate) const fn internal_result<T>(message: String) -> Result<T, ErrorWrapper>  {
        return Err(ErrorWrapper::new_internal(message));
    }
    
    pub(crate) fn io_error_result<T>(error: std::io::Error) -> Result<T, ErrorWrapper> {
        Err(ErrorWrapper::from(error))
    }
}

impl From<std::io::Error> for ErrorWrapper {
    fn from(io_error: std::io::Error) -> ErrorWrapper {
        ErrorWrapper { message: None, io_error: Some(io_error),  error_type: IO }
    }
}

impl Debug for ErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            INTERNAL => {
                write!(f, "{}", self.message.as_ref().unwrap())
            },
            IO => {
                write!(f, "{}", self.io_error.as_ref().unwrap())
            }
        }
    }
}

impl Display for ErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error_type= &self.error_type;
        match error_type {
            INTERNAL => {
                write!(f, "{}", self.message.as_ref().unwrap())
            },
            IO => {
                write!(f, "{}", self.io_error.as_ref().unwrap())
            }
        }  
    }
}

pub fn error_result<T>(message: String) -> Result<T, ErrorWrapper> {
    return Err(ErrorWrapper::new_internal(message));
}