use std::any::Any;
use std::collections::VecDeque;
use std::io;
use termion::event::Key;
use termion::input::TermRead;

pub trait KeyInputResolver {
    fn get_input_key(&mut self) -> Result<Key, io::Error>;
    fn get_or_return_input_key(&mut self, input : Option<Key>) -> Result<Key, io::Error>;

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Clone)]
pub struct IoKeyInputResolver {}

fn get_input_key() -> Result<Key, io::Error> {
    io::stdin().keys().next().unwrap()
}

impl KeyInputResolver for IoKeyInputResolver {
    fn get_input_key(&mut self) -> Result<Key, io::Error> {
        Ok(get_input_key()?)
    }
    
    fn get_or_return_input_key(&mut self, input : Option<Key>) -> Result<Key, io::Error> {
        match input {
            Some(input_key) => {
                Ok(input_key)
            },
            _ => {
                Ok(get_input_key()?)
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct MockKeyInputResolver {
    pub key_results : VecDeque<Key>
}

impl KeyInputResolver for MockKeyInputResolver {
    fn get_input_key(&mut self) -> Result<Key, io::Error> {
        Ok(self.key_results.pop_front().unwrap().clone())
    }

    fn get_or_return_input_key(&mut self, input : Option<Key>) -> Result<Key, io::Error> {
        match input {
            // Return the provided key if it exists
            Some(input_key) => {
                Ok(input_key)
            },
            _ => {
                // Otherwise fetch a new one
                self.get_input_key()
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}