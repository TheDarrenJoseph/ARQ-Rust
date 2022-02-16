use std::io;
use termion::event::Key;
use std::collections::HashMap;
use termion::input::TermRead;
use std::collections::HashSet;

use crate::view::framehandler::container_view;
use crate::engine::command::command::Command;
use crate::view::world_container_view::{WorldContainerViewFrameHandler, WorldContainerView};
use crate::view::framehandler::container_view::{ContainerViewInputResult, ContainerViewCommand};
use crate::view::framehandler::container_view::ContainerViewInputResult::{TAKE_ITEMS, DROP_ITEMS};
use crate::map::position::Position;
use crate::view::callback::Callback;
use crate::view::View;
use crate::map::Map;
use crate::engine::level::Level;
use crate::ui;
use crate::map::objects::container::Container;
use crate::engine::command::input_mapping;
use crate::terminal::terminal_manager::TerminalManager;
use crate::view::framehandler::container_view::ContainerViewCommand::{OPEN, TAKE, DROP};

pub struct OpenCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

fn handle_callback(level : &mut Level, container: &mut Container, data : ContainerViewInputResult) -> Option<ContainerViewInputResult> {
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
        }
        _ => {}
    }
    return None
}

impl <B: tui::backend::Backend> OpenCommand<'_, B> {

    // TODO refactor alongside other commands / engine func
    fn re_render(&mut self) -> Result<(), io::Error>  {
        let mut ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn open_container(&mut self, c: &Container) -> Container {
        log::info!("Player opening container.");
        let mut subview_container = c.clone();
        let mut view_container = c.clone();
        let mut callback_container : Container = c.clone();
        let mut commands : HashSet<ContainerViewCommand> = HashSet::new();
        commands.insert(OPEN);
        commands.insert(TAKE);
        let mut container_view = container_view::build_container_view(subview_container, commands);

        let ui = &mut self.ui;
        let terminal_manager = &mut self.terminal_manager;
        let frame_handler = WorldContainerViewFrameHandler { container_views: vec![container_view] };

        let level = &mut self.level;
        let mut world_container_view = WorldContainerView {
            ui,
            terminal_manager,
            frame_handler,
            container: view_container,
            callback: Box::new(|data| {None})
        };

        world_container_view.set_callback(Box::new(|data| {
            handle_callback(level, &mut callback_container, data)
        }));
        world_container_view.begin();
        world_container_view.container.clone()
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
        if let Some(p) = self.level.find_adjacent_player_position(key, command_key) {
            log::info!("Player opening at map position: {}, {}", &p.x, &p.y);
            self.re_render();

            let mut updated_container = None;
            let mut target_position = None;

            let mut to_open = None;
            if let Some(map) = &mut self.level.map {
                if let Some(room) = map.get_rooms().iter_mut().find(|r| r.area.contains_position(p)) {
                    if let Some(c) = room.containers.get(&p) {
                        log::info!("Found room container.");
                        target_position = Some(p.clone());
                        to_open = Some(c.clone());
                    } else if let Some(c) = map.containers.get(&p) {
                        if c.get_item_count() > 0 {
                            log::info!("Found map container.");
                            target_position = Some(p.clone());
                            to_open = Some(c.clone());
                        }
                    } else if let Some(door) = &room.doors.iter().find(|d| d.position == p) {
                        log::info!("Player opening door.");
                        self.ui.console_print("There's a door here.".to_string());
                        // TODO encapsulate view components / refactor
                        self.re_render();
                    } else {
                        self.ui.console_print("There's nothing here to open.".to_string());
                        // TODO encapsulate view components / refactor
                        self.re_render();
                    }
                }
            }

            if let Some(c) = to_open {
                log::info!("Player opening container of type {:?} and length: {}", c.container_type, c.get_item_count());
                updated_container = Some(self.open_container(&c));
            }

            // Replace the original container with any changes we've made
            if let Some(container) = updated_container {
                if let Some(pos) = target_position {
                    if let Some(map) = &mut self.level.map {
                        if let Some(original_room) = map.rooms.iter_mut().find(|r| {
                            r.area.contains_position(pos) &&
                                if let Some(room_container) = r.containers.get(&pos) {
                                    room_container.id_equals(&container) }
                                else {
                                    false
                                }
                        }) {
                            log::info!("Updating room container with changes..");
                            original_room.containers.insert(p, container);
                        } else if let Some(area_container) = map.containers.get(&pos) {
                            log::info!("Updating area container with changes..");
                            map.containers.insert(p, container);
                        }
                    }
                }
            }

            let key = io::stdin().keys().next().unwrap().unwrap();
        }
        Ok(())
    }
}