use std::io;
use termion::event::Key;
use std::collections::HashMap;
use termion::input::TermRead;

use crate::view::framehandler::container_view;
use crate::engine::command::command::Command;
use crate::view::world_container_view::{WorldContainerViewFrameHandler, WorldContainerView};
use crate::view::framehandler::container_view::ContainerViewInputResult;
use crate::view::framehandler::container_view::ContainerViewInputResult::TAKE_ITEMS;
use crate::map::position::Position;
use crate::view::callback::Callback;
use crate::view::View;
use crate::map::Map;
use crate::engine::level::Level;
use crate::ui;
use crate::map::objects::container::Container;
use crate::engine::command::input_mapping;
use crate::terminal::terminal_manager::TerminalManager;

pub struct OpenCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

fn handle_take_items(level : &mut Level, container: &mut Container, data : ContainerViewInputResult) {
    let input_result : ContainerViewInputResult = data;
    match input_result {
        TAKE_ITEMS(items) => {
            let player = &mut level.characters[0];
            log::info!("Received data for TAKE_ITEMS with {} items", items.len());
            log::info!("Found player: {}", player.get_name());
            for item in items {
                if let Some(container_item) = container.find_mut(&item) {
                    player.get_inventory().add(container_item.clone());
                }
            }
        },
        _ => {}
    }
}

impl <B: tui::backend::Backend> OpenCommand<'_, B> {
    fn find_adjacent_player_position(&mut self, key: Key, command_char: Key) -> Option<Position> {
        return match key {
            Key::Down | Key::Up | Key::Left | Key::Right => {
                if let Some(side) = input_mapping::key_to_side(key) {
                    self.level.find_player_side_position(side)
                } else {
                    None
                }
            },
            command_char => {
                Some(self.level.get_player_mut().get_position().clone())
            }
            _ => {
                None
            }
        };
    }
}

impl <B: tui::backend::Backend> Command for OpenCommand<'_, B> {
    fn handles_key(&self, key: Key) -> bool {
        return match key {
            Key::Char('o') => {
                true
            }
            _ => {
                false
            }
        };
    }

    fn handle(&mut self, command_key: Key) -> Result<(), io::Error> {

        let key = io::stdin().keys().next().unwrap().unwrap();
        if let Some(p) = self.find_adjacent_player_position(key, command_key) {
            log::info!("Player opening at map position: {}, {}", &p.x, &p.y);
            //self.re_render();

            let mut updated_container = None;
            let mut target_position = None;
            let map = &mut self.level.map;
            if let Some(map) = map {
                if let Some(room) =  map.get_rooms().iter_mut().find(|r| r.area.contains_position(p)) {
                    if let Some(c) = room.containers.get(&p) {
                        log::info!("Player opening container.");
                        target_position = Some(p.clone());
                        let mut inventory_container = c.clone();
                        let mut view_container = c.clone();
                        let mut callback_container : Container = c.clone();
                        let mut container_view = container_view::build_container_view( inventory_container);

                        let ui = &mut self.ui;
                        let level = &mut self.level;
                        let terminal_manager = &mut self.terminal_manager;
                        let frame_handler = WorldContainerViewFrameHandler { container_views: vec![container_view] };
                        let mut world_container_view = WorldContainerView {
                            ui,
                            terminal_manager,
                            frame_handler,
                            container: view_container,
                            callbacks: HashMap::new()
                        };

                        let open_callback = Box::new(|data| {
                            handle_take_items(level, &mut callback_container, data);
                        });
                        world_container_view.set_callback(String::from("t"), open_callback);
                        world_container_view.begin();

                        updated_container = Some(world_container_view.container.clone());

                    } else if let Some(door) = &room.doors.iter().find(|d| d.position == p) {
                        log::info!("Player opening door.");
                        self.ui.console_print("There's a door here.".to_string());
                        // TODO encapsulate view components / refactor
                        //self.re_render();
                    } else {
                        self.ui.console_print("There's nothing here to open.".to_string());
                        // TODO encapsulate view components / refactor
                        //self.re_render();
                    }
                }
            }

            // Replace the original container with any changes we've made
            if let Some(container) = updated_container {
                if let Some(pos) = target_position {
                    if let Some(map) = &mut self.level.map {
                        if let Some(original_room) = map.rooms.iter_mut().find(|r| r.area.contains_position(pos)) {
                            original_room.containers.insert(p, container);
                        }
                    }
                }
            }

            let key = io::stdin().keys().next().unwrap().unwrap();
        }
        Ok(())
    }
}