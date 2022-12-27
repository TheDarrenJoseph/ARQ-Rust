use std::collections::HashSet;
use crate::view::map::MapView;
use crate::view::usage::UsageLine;

#[derive(Eq, Hash, PartialEq)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum MapViewCommand {
    INVENTORY
}

impl <B: tui::backend::Backend> UsageLine<MapViewCommand> for MapView<'_, B> {
    fn build_command_usage_descriptions(commands: &Vec<MapViewCommand>) -> String {
        let mut description = String::from("");
        let len = commands.len();
        let mut i = 0;
        for c in commands.iter() {
            match c {
                // Inventory command launches the Character Info view, so show that here
                MapViewCommand::INVENTORY => {
                    description += "i - Character Info";
                }
            }
            if i < len - 1 {
                description += ", ";
            }
            i+=1;
        }
        description
    }
}