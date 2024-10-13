use crate::ui::bindings::input_bindings::KeyBindings;
use std::collections::HashMap;
use termion::event::Key;

/*
  An Action that the Player can take
 */
pub enum Action {
    ShowInventory,
    LookAround,
    OpenNearby,
    Escape // This can open the pause menu, close a container view, etc
}

pub struct ActionKeyBindings {
   pub bindings : HashMap<Key, Action>
}

impl KeyBindings<Action> for ActionKeyBindings {
    fn get_bindings(&self) -> &HashMap<Key, Action> {
       &self.bindings
    }

    fn get_input(&self, key: Key) -> Option<&Action> {
        self.get_bindings().get(&key)
    }
}


