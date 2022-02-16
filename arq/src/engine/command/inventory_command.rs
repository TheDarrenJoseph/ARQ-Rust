use std::io;
use termion::event::Key;
use std::collections::HashMap;
use termion::input::TermRead;
use std::collections::HashSet;

use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::view::View;
use crate::view::framehandler::container_view::{ContainerViewInputResult, ContainerViewCommand};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui;
use crate::view::framehandler::container_view::ContainerViewInputResult::{DROP_ITEMS, TAKE_ITEMS};
use crate::view::framehandler::container_view::ContainerViewCommand::{OPEN, TAKE, DROP};
use crate::view::character_info_view::{CharacterInfoViewFrameHandler, CharacterInfoView, TabChoice};
use crate::engine::command::command::Command;
use crate::view::callback::Callback;

pub struct InventoryCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

fn handle_callback(level : &mut Level, container: &mut Container, data : ContainerViewInputResult) -> Option<ContainerViewInputResult> {
    let input_result : ContainerViewInputResult = data;
    match input_result {
        DROP_ITEMS(items) => {
            let position = level.get_player_mut().get_position().clone();
            log::info!("InventoryCommand - Dropping {} items at position: {}, {}", items.len(),  position.x, position.y);

            // Find the container on the map and add the "container" wrappers there
            let mut undropped = Vec::new();
            if let Some(m) = level.get_map_mut() {
                if let Some(mut pos_container) = m.get_container_mut(position) {
                    for item in items {
                        // Find the "container" wrappper matching the item returned
                        if let Some(container_item) = container.find_mut(&item) {
                            let dropping_container_item = container_item.clone();
                            if pos_container.can_fit_container_item(&dropping_container_item) {
                                log::info!("Dropping item here: {}", item.get_name());
                                pos_container.add(dropping_container_item)
                            } else {
                                log::info!("Cannot fit item here: {}", item.get_name());
                                undropped.push(item);
                            }
                        }
                    }
                }
            }
            return Some(DROP_ITEMS(undropped));
        }
        _ => {}
    }
    None
}

impl <B: tui::backend::Backend> InventoryCommand<'_, B> {

    fn re_render(&mut self) -> Result<(), io::Error>  {
        let mut ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn open_inventory(&mut self) {
        log::info!("Player opening inventory.");
        let c = self.level.get_player_mut().get_inventory();
        let mut inventory_container = c.clone();
        let mut view_container = c.clone();
        let mut callback_container: Container = c.clone();
        let mut commands: HashSet<ContainerViewCommand> = HashSet::new();

        let ui = &mut self.ui;
        let terminal_manager = &mut self.terminal_manager;
        let frame_handler = CharacterInfoViewFrameHandler { tab_choice: TabChoice::INVENTORY, container_views: Vec::new(), character_view: None };

        self.ui.hide_console();
        let level = &mut self.level;
        let player = &mut level.characters[0].clone();
        let updated_inventory;
        {
            let mut character_info_view = CharacterInfoView { character: player, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, frame_handler, callback: Box::new(|data| {None}) };
            character_info_view.set_callback(Box::new(|data| {
                handle_callback(level, &mut callback_container, data)
            }));
            character_info_view.begin();
            updated_inventory = character_info_view.frame_handler.container_views.get(0).unwrap().container.clone();
        }
        level.characters[0].set_inventory(updated_inventory);
        self.ui.show_console();
    }
}

impl <B: tui::backend::Backend> Command for InventoryCommand<'_, B> {
    fn handles_key(&self, key: Key) -> bool {
        return match key {
            Key::Char('i') => {
                true
            }
            _ => {
                false
            }
        };
    }

    fn handle(&mut self, command_key: Key) -> Result<(), io::Error> {
        self.open_inventory();
        Ok(())
    }
}