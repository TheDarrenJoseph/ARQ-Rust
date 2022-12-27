pub mod container_framehandler_usage;
pub mod map_view_usage;

use std::collections::HashSet;
use termion::event::Key;
use crate::character::Character;

#[derive(Eq, Hash, PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct UsageCommand  {
    pub key : char,
    pub description : String
}

impl UsageCommand {
    pub fn get_key(self) -> char {
        self.key
    }

    pub fn get_description(self) -> String {
        self.description
    }

    fn describe_usage(&self) -> String {
        format!("{} - {}", self.key, self.description)
    }
}

pub fn build_usage(key: char, description: String) -> UsageCommand {
    UsageCommand { key, description}
}

pub trait UsageLine<COM> {
    fn build_command_usage_descriptions(commands: &Vec<COM>) -> String;
}