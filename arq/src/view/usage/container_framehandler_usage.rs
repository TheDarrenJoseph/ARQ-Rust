use std::collections::HashSet;
use termion::event::Key;

use crate::view::framehandler::container::ContainerFrameHandler;
use crate::view::usage::{UsageCommand, UsageLine};

impl UsageLine<UsageCommand> for ContainerFrameHandler {
    fn build_command_usage_descriptions(commands: &Vec<UsageCommand>) -> String {
        let mut description = String::from("");
        let len = commands.len();
        let mut i = 0;
        for c in commands.iter() {
            description += c.describe_usage().as_str();
            if i < len - 1 {
                description += ", ";
            }
            i+=1;
        }
        description
    }
}

