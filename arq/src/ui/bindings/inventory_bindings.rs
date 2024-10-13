use crate::ui::bindings::input_bindings::KeyBindings;
use std::collections::HashMap;
use termion::event::Key;

pub enum InventoryInput {
}

pub struct InventoryKeyBindings {
    pub bindings : HashMap<Key, InventoryInput>
}

impl KeyBindings<InventoryInput> for InventoryKeyBindings {
    fn get_bindings(&self) -> &HashMap<Key, InventoryInput> {
        &self.bindings
    }

    fn get_input(&self, key: Key) -> Option<&InventoryInput> {
        self.get_bindings().get(&key)
    }
}
