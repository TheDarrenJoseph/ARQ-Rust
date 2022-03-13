use std::io;
use termion::event::Key;
use termion::input::TermRead;

use crate::view::framehandler::container;
use crate::engine::command::command::Command;
use crate::view::world_container::{WorldContainerViewFrameHandlers, WorldContainerView};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult;
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::TAKE_ITEMS;
use crate::map::position::Position;
use crate::view::callback::Callback;
use crate::view::View;
use crate::map::Map;
use crate::engine::level::Level;
use crate::ui;
use crate::map::objects::container::Container;
use crate::engine::command::input_mapping;
use crate::terminal::terminal_manager::TerminalManager;

pub struct LookCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

impl <B: tui::backend::Backend> LookCommand<'_, B> {
    // TODO refactor alongside other commands / engine func
    fn re_render(&mut self) -> Result<(), io::Error> {
        let mut ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }
}

impl <B: tui::backend::Backend> Command for LookCommand<'_, B> {
    fn handles_key(&self, key: Key) -> bool {
        return match key {
            Key::Char('k') => {
                true
            }
            _ => {
                false
            }
        };
    }

    fn handle(&mut self, command_key: Key) -> Result<(), io::Error> {
        self.ui.console_print("Where do you want to look?. Arrow keys to choose. Repeat command to choose current location.".to_string());
        self.re_render();
        let key = io::stdin().keys().next().unwrap().unwrap();
        let position = self.level.find_adjacent_player_position(key, command_key);

        if let Some(p) = position {
            log::info!("Player looking at map position: {}, {}", &p.x, &p.y);
            self.re_render();

            if let Some(map) = &self.level.map {
                if let Some(room) =  map.get_rooms().iter().find(|r| r.area.contains_position(p)) {
                    log::info!("Position is in a room.");

                    if let Some(c) = &room.containers.get(&p) {
                        log::info!("Position is a container.");
                        let container_item = c.get_self_item();
                        self.ui.console_print("There's a ".to_owned() + &container_item.name + &" here.".to_string());
                        self.re_render();
                    } else if let Some(door) = &room.doors.iter().find(|d| d.position == p) {
                        log::info!("Position is a door.");
                        self.ui.console_print("There's a ".to_owned() + &door.tile_details.name + &" here.".to_string());
                        self.re_render();
                    } else {
                        self.ui.console_print("There's nothing here in this room.".to_string());
                        self.re_render();
                    }
                }
            }

            let key = io::stdin().keys().next().unwrap().unwrap();
        }
        Ok(())
    }
}