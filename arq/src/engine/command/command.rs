use std::io;

use termion::event::Key;
use crate::engine::command::input_bindings::{Action, Input};
use crate::error::errors::ErrorWrapper;

/*
 A command is a way for an action to be executed upon the game state
 */
pub trait Command {
    fn can_handle_action(&self, action: Action) -> bool;
    fn handle_input(&mut self, input: Option<Input>) -> Result<(), ErrorWrapper>;
}
