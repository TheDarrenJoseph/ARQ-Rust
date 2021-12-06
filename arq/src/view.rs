use std::io::Error;

pub mod character_view;
pub mod container_view;
pub mod map_view;

pub trait View {
    fn draw(&mut self) -> Result<(), Error>;
    fn handle_input(&mut self) -> Result<bool, Error>;
}
