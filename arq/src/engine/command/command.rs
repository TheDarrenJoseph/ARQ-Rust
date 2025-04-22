use crate::error::errors::ErrorWrapper;
use crate::ui::bindings::action_bindings::Action;
/*
 A command is a way for an action to be executed upon the game state
 */
pub trait Command<Input> {
    fn can_handle_action(&self, action: Action) -> bool;

    fn start(&mut self) -> Result<(), ErrorWrapper>;

    fn handle_input(&mut self, input: Option<&Input>) -> Result<(), ErrorWrapper>;
}