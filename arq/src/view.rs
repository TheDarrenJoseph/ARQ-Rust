use std::io::Error;

pub trait View {
    fn draw(&mut self) -> Result<(), Error>;
    fn handle_input(&mut self) -> Result<bool, Error>;
}