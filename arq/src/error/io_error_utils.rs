use std::io::{Error, ErrorKind};


pub fn error(msg: String) -> std::io::Error {
    return Error::new(ErrorKind::Other, msg);
}

pub fn error_result<T>(msg: String) -> Result<T, Error> {
    return Err(error( msg));
}