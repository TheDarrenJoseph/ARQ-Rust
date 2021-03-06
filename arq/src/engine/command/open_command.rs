use std::io;
use termion::event::Key;
use std::collections::HashMap;
use termion::input::TermRead;
use std::collections::HashSet;

use crate::view::framehandler::container;
use crate::engine::command::command::Command;
use crate::view::world_container::{WorldContainerViewFrameHandlers, WorldContainerView};
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, ContainerFrameHandlerCommand};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{TakeItems, DropItems, MoveItems};
use crate::map::position::Position;
use crate::view::callback::Callback;
use crate::view::View;
use crate::map::Map;
use crate::engine::level::Level;
use crate::ui;
use crate::map::objects::container::Container;
use crate::engine::command::input_mapping;
use crate::engine::container_util;
use crate::map::objects::items::Item;
use crate::terminal::terminal_manager::TerminalManager;
use crate::view::framehandler::container::ContainerFrameHandlerCommand::{OPEN, TAKE, DROP};
use crate::engine::container_util::*;

pub struct OpenCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>
}

fn handle_callback(level : &mut Level, position: Position, container: Container, data : ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> {
    let input_result : ContainerFrameHandlerInputResult = data;
    match input_result {
        TakeItems(mut data) => {
            log::info!("[open command] Received data for TakeItems with {} items", data.to_take.len());
            data.position = Some(position.clone());
            return container_util::take_items(data , level);
        },
        MoveItems(mut data) => {
            log::info!("[open command] Received data for MoveItems with {} items", data.to_move.len());
            data.position = Some(position.clone());
            return container_util::move_items(data, level);
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

    fn open_container(&mut self, p: Position, c: &Container) {
        log::info!("Player opening container: {} at position: {:?}", c.get_self_item().get_name(), p);
        let callback_container : Container = c.clone();
        let mut subview_container = c.clone();
        let mut view_container = c.clone();

        let mut commands : HashSet<ContainerFrameHandlerCommand> = HashSet::new();
        commands.insert(OPEN);
        commands.insert(TAKE);
        let mut container_view = container::build_container_view(subview_container, commands);

        let ui = &mut self.ui;
        let terminal_manager = &mut self.terminal_manager;
        let frame_handler = WorldContainerViewFrameHandlers { frame_handlers: vec![container_view] };
        let level = &mut self.level;
        let mut world_container_view = WorldContainerView {
            ui,
            terminal_manager,
            frame_handlers: frame_handler,
            container: view_container,
            callback: Box::new(|data| {None})
        };
        world_container_view.set_callback(Box::new(|input_result| {
            return handle_callback(level, p.clone(), callback_container.clone(), input_result);
        }));
        world_container_view.begin();

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
                    if let Some(door) = &room.doors.iter().find(|d| d.position == p) {
                        log::info!("Player opening door.");
                        self.ui.console_print("There's a door here.".to_string());
                        // TODO encapsulate view components / refactor
                    } else {
                        self.ui.console_print("There's nothing here to open.".to_string());
                        // TODO encapsulate view components / refactor
                    }
                }

                if let None = to_open  {
                    if let Some(c) = map.containers.get(&p) {
                        let item_count = c.get_content_count();
                         if item_count > 0 {
                             log::info!("Found map container.");
                             target_position = Some(p.clone());
                             let contains_single_container = item_count == 1 && c.get_contents()[0].is_true_container();
                             if contains_single_container &&  c.get_contents()[0].get_content_count() > 0 {
                                 to_open = Some(c.get_contents()[0].clone());
                             } else {
                                 to_open = Some(c.clone());
                             }
                        }
                    } else {
                        self.ui.console_print("There's nothing here to open.".to_string());
                        // TODO encapsulate view components / refactor
                        self.re_render();
                    }
                }
            }

            if let Some(c) = to_open {
                self.re_render();
                log::info!("Player opening container of type {:?} and length: {}", c.container_type, c.get_item_count());
                updated_container = Some(self.open_container(p.clone(), &c));
            }

            let key = io::stdin().keys().next().unwrap().unwrap();
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use std::collections::HashMap;
    use tui::backend::TestBackend;
    use std::collections::HashSet;
    use termion::input::TermRead;
    use tui::text::Text;
    use tui::layout::Rect;
    use tui::buffer::{Buffer, Cell};
    use tui::widgets::Widget;

    use crate::ui;
    use crate::terminal;
    use crate::map::objects::container;
    use crate::map::objects::container::{build, ContainerType, Container};
    use crate::map::objects::items;
    use crate::menu;
    use crate::view::framehandler::container::{ContainerFrameHandler, build_container_view, build_default_container_view, Column, ContainerFrameHandlerInputResult, TakeItemsData};
    use crate::terminal::terminal_manager::TerminalManager;
    use crate::ui::{UI, build_ui};
    use crate::list_selection::ListSelection;
    use crate::view::framehandler::console::{ConsoleFrameHandler, ConsoleBuffer};
    use crate::map::tile::{Colour, Tile};
    use crate::engine::command::open_command::{OpenCommand, handle_callback};
    use crate::engine::level::Level;
    use crate::character::{build_player, Character, build_default_character_details, build_character};
    use crate::map::position::{Position, build_square_area};

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container = build(id, "Test Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container_self_item = container.get_self_item();
        assert_eq!(id, container_self_item.get_id());
        assert_eq!("Test Container", container_self_item.name);
        assert_eq!('X', container_self_item.symbol);
        assert_eq!(Colour::White, container_self_item.colour);
        assert_eq!(1, container_self_item.weight);
        assert_eq!(1, container_self_item.value);

        for i in 1..=4 {
            let test_item = items::build_item(Uuid::new_v4(), format!("Test Item {}", i), 'X', 1, 100);
            container.add_item(test_item);
        }

        assert_eq!(ContainerType::OBJECT, container.container_type);
        assert_eq!(100, container.get_weight_limit());
        let contents = container.get_contents();
        assert_eq!(4, contents.len());
        container
    }

    fn build_test_level(player: Character) -> Level {
        let tile_library = crate::map::tile::build_library();
        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();
        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);

        let map = crate::map::Map {
            area: map_area,
            tiles: vec![
                vec![wall.clone(), wall.clone(), wall.clone()],
                vec![wall.clone(), rom.clone(), wall.clone()],
                vec![wall.clone(), wall.clone(), wall.clone()],
            ],
            rooms: Vec::new(),
            containers: HashMap::new()
        };

        return Level { map: Some(map), characters: vec![player] };
    }

    #[test]
    fn test_take_callback() {
        // GIVEN a valid level with an player inventory to extract items into
        let inventory = build(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 2);
        let character_details = build_default_character_details();
        let player = build_character(String::from("Test Player") , Position { x: 0, y: 0}, inventory);
        let mut level = build_test_level(player);
        let container_pos =  Position { x: 0, y: 0};

        // WHEN we call to handle a take callback with some of the items in a container
        let mut container = build_test_container();
        let mut selected_container_items = Vec::new();
        for i in 0..=1 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(2, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();

        let data = TakeItemsData { source: container.clone(), to_take: selected_container_items, position: Some(container_pos) };
        let mut view_result = ContainerFrameHandlerInputResult::TakeItems(data);
        let untaken = handle_callback(&mut level, container_pos, container, view_result).unwrap();

        // THEN we expect a DropItems returned with 0 un-taken items
        match untaken {
            ContainerFrameHandlerInputResult::TakeItems(data) => {
                assert_eq!(0, data.to_take.len());
            },
            _ => {
                assert!(false);
            }
        }

        // AND we expect the inventory to contain the 2 items taken
        let inventory = level.get_player_mut().get_inventory();
        let updated_container_contents = inventory.get_contents();
        assert_eq!(2, updated_container_contents.len());
        assert_eq!(chosen_item_1, *updated_container_contents.get(0).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_container_contents.get(1).unwrap().get_self_item());
    }

    #[test]
    fn test_take_callback_too_many_items() {
        // GIVEN a valid map with an player inventory to extract items into
        let inventory = build(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 2);
        let character_details = build_default_character_details();
        let player = build_character(String::from("Test Player") , Position { x: 0, y: 0}, inventory);
        let mut level = build_test_level(player);
        let container_pos =  Position { x: 0, y: 0};

        // WHEN we call to handle a take callback with 3 items (with only space for 2 of them)
        let mut container = build_test_container();
        let mut callback_container = container.clone();
        let mut selected_container_items = Vec::new();
        for i in 0..=2 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(3, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        let chosen_item_3 = selected_container_items.get(2).unwrap().clone();
        let data = TakeItemsData { source: container, to_take: selected_container_items, position: Some(container_pos) };
        let mut view_result = ContainerFrameHandlerInputResult::TakeItems(data);
        let untaken = handle_callback(&mut level, container_pos, callback_container, view_result).unwrap();

        // THEN we expect a DropItems returned with 1 un-taken items
        match untaken {
            ContainerFrameHandlerInputResult::TakeItems(u) => {
                assert_eq!(1, u.to_take.len());
                assert_eq!(chosen_item_3, *u.to_take.get(0).unwrap());
            },
            _ => {
                assert!(false);
            }
        }

        // AND we expect the inventory to contain the 2 items taken
        let inventory = level.get_player_mut().get_inventory();
        let updated_container_contents = inventory.get_contents();
        assert_eq!(2, updated_container_contents.len());
        assert_eq!(chosen_item_1, *updated_container_contents.get(0).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_container_contents.get(1).unwrap().get_self_item());
    }

}