

use crate::map::position::Position;

#[derive(Eq, Hash, PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct UsageCommand  {
    pub key : char,
    pub description : String
}

impl UsageCommand {
    pub const fn new(key: char, description: String) -> Self {
        UsageCommand { key, description}
    }

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

#[derive(Clone)]
#[derive(Debug)]
pub struct UsageLine {
    pub commands : Vec<UsageCommand>,
    pub start_position: Option<Position>
}

impl UsageLine {
    pub const fn new(commands: Vec<UsageCommand>) -> Self {
        UsageLine { commands, start_position: None }
    }

    pub fn describe(&self) -> String {
        let mut description = String::from("");
        let len = self.commands.len();
        let mut i = 0;
        for c in self.commands.iter() {
            description += c.describe_usage().as_str();
            if i < len - 1 {
                description += ", ";
            }
            i += 1;
        }
        description
    }
}
