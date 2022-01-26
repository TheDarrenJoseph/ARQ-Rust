use std::io;
use termion::event::Key;

pub trait Command {
    fn handles_key(&self, key: Key) -> bool;
    fn handle(&mut self, command_key: Key) -> Result<(), io::Error>;
}
