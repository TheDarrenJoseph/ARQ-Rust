pub mod errors;
pub mod io_error_utils;

// TODO hookup for messages
pub struct ErrorWrapper {
    message: String,
    io_error: std::io::Error
}
