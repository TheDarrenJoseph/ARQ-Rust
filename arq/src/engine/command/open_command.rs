use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{Error, ErrorKind};
use log::error;

use termion::event::Key;
use termion::input::TermRead;

use crate::engine::command::command::Command;
use crate::engine::container_util;
use crate::engine::level::Level;
use crate::error::io_error_utils::error_result;
use crate::map::objects::container::Container;
use crate::map::position::Position;
use crate::terminal::terminal_manager::TerminalManager;
use crate::view::util::callback::Callback;
use crate::view::framehandler::container;
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, MoveItemsData, MoveToContainerChoiceData};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{MoveItems, MoveToContainerChoice, TakeItems};
use crate::view::{InputResult, View};
use crate::view::world_container::{WorldContainerView, WorldContainerViewFrameHandlers};
use crate::ui::ui::{get_input_key, UI};
use crate::view::model::usage_line::{UsageCommand, UsageLine};

pub struct OpenCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>
}

const NOTHING_ERROR : &str = "There's nothing here to open.";

fn handle_callback<'a>(level : &'a mut Level, position: Position, data : ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> {
    let mut input_result : ContainerFrameHandlerInputResult = data;
    match input_result {
        TakeItems(mut data) => {
            log::info!("[open usage] Received data for TakeItems with {} items", data.to_take.len());
            data.position = Some(position.clone());
            return container_util::take_items(data , level);
        },
        MoveItems(mut data) => {
            log::info!("[open usage] Received data for MoveItems with {} items", data.to_move.len());
            data.position = Some(position.clone());
            return container_util::move_items(data, level);
        },
        MoveToContainerChoice(ref mut data) => {
            if let Some(_target) = &data.target_container {
                // Translate to the typical moving data
                let move_data = MoveItemsData {
                    source: data.source.clone(),
                    to_move: data.to_move.clone(),
                    target_container: data.target_container.clone(),
                    target_item: None,
                    // Both position and a target needed to move to a world container
                    position: Some(position)
                };
                log::info!("[open usage] Moving items for MoveToContainerChoice...");
                return container_util::move_items(move_data, level);
            } else {
                log::info!("[open usage] Building choices for MoveToContainerChoice...");
                // Add the position of the currently opened container
                data.position = Some(position);
                // Build container choices and pass the result back down to the view/handlers
                let choices_result = build_container_choices(data, level);
                if choices_result.is_ok() {
                    return choices_result.ok();
                } else {
                    log::error!("{}", choices_result.err().unwrap());
                }
            }
        }
        _ => {}
    }
    return None
}

fn build_container_choices<'a>(data: &'a MoveToContainerChoiceData, level: &'a mut Level) -> Result<ContainerFrameHandlerInputResult, Error> {
    if let Some(pos) = data.position {
        let map = level.get_map_mut().unwrap();
        let container_result = map.find_container_mut(pos);
        if let Some(container) = container_result {
            let sub_containers_result = container_util::build_container_choices(&data.source, container);
            if sub_containers_result.is_ok() {
                let choices = sub_containers_result.unwrap();
                log::error!("Built {} sub-container choices for position: {:?}", choices.len(), pos);
                let mut result_data = data.clone();
                result_data.choices = choices;
                result_data.position = Some(pos);
                return Ok(MoveToContainerChoice(result_data));
            } else {
                return error_result(format!("Failed build container choices: {}", sub_containers_result.err().unwrap()));
            }
        } else {
            return error_result( format!("Cannot build container choices. Cannot find container at position: {:?}", pos));
        }
    } else {
        return error_result(String::from("Cannot build container choices. No position provided."));
    }
}

impl <B: tui::backend::Backend> OpenCommand<'_, B> {

    // TODO refactor alongside other commands / engine func
    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn open_container(&mut self, p: Position, c: &Container) -> Result<InputResult<bool>, Error> {
        log::info!("Player opening container: {} at position: {:?}", c.get_self_item().get_name(), p);
        let subview_container = c.clone();
        let view_container = c.clone();

        let mut commands : HashMap<Key, UsageCommand> = HashMap::new();
        commands.insert(Key::Char('o'), UsageCommand::new('o', String::from("open") ));
        commands.insert(Key::Char('t'), UsageCommand::new('t', String::from("take")) );
        let usage_line = UsageLine::new(commands);
        let container_view = container::build_container_frame_handler(subview_container, usage_line);

        let ui = &mut self.ui;
        let terminal_manager = &mut self.terminal_manager;
        let frame_handler = WorldContainerViewFrameHandlers { container_frame_handlers: vec![container_view], choice_frame_handler: None };
        let level = &mut self.level;
        let mut world_container_view = WorldContainerView {
            ui,
            terminal_manager,
            frame_handlers: frame_handler,
            container: view_container,
            callback: Box::new(|_data| {None})
        };
        world_container_view.set_callback(Box::new(|input_result| {
            return handle_callback(level, p.clone(), input_result);
        }));
        world_container_view.begin()
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

    fn handle(&mut self, key: Key) -> Result<(), io::Error> {

        let mut message = NOTHING_ERROR.to_string();
        if let Some(p) = self.level.find_adjacent_player_position(key) {
            log::info!("Player opening at map position: {}, {}", &p.x, &p.y);
            self.re_render()?;

            let mut to_open = None;
            if let Some(map) = &mut self.level.map {
                if let Some(room) = map.get_rooms().iter_mut().find(|r| r.get_area().contains_position(p)) {
                    if let Some(_door) = &room.get_doors().iter().find(|d| d.position == p) {
                        log::info!("Player opening door.");
                        message = "There's a door here.".to_string();
                    }
                }

                if let None = to_open {
                    if let Some(c) = map.containers.get(&p) {
                        let item_count = c.get_top_level_count();
                        if item_count > 0 {
                            log::info!("Found map container.");
                            let contains_single_container = item_count == 1 && c.get_contents()[0].is_true_container();
                            if contains_single_container && c.get_contents()[0].get_top_level_count() > 0 {
                                to_open = Some(c.get_contents()[0].clone());
                            } else {
                                to_open = Some(c.clone());
                            }
                        }
                    }
                }
            }

            if let Some(c) = to_open {
                self.re_render()?;
                log::info!("Player opening container of type {:?} and length: {}", c.container_type, c.get_total_count());
                self.open_container(p.clone(), &c)?;
            } else {
                self.ui.console_print(message);
                self.re_render();
                io::stdin().keys().next().unwrap()?;
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use uuid::Uuid;

    use crate::character::Character;
    use crate::character::character_details::build_default_character_details;
    use crate::character::characters::{build_characters, build_default_characters, Characters};
    use crate::engine::command::open_command::{handle_callback};
    use crate::engine::level::{Level};

    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::objects::items;
    use crate::map::objects::items::Item;
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::{Colour, Symbol, Tile};
    use crate::map::Tiles;


    use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, TakeItemsData};

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container = build(id, "Test Container".to_owned(), 'X', 1.0, 1, ContainerType::OBJECT, 100);
        let container_self_item = container.get_self_item();
        assert_eq!(id, container_self_item.get_id());
        assert_eq!("Test Container", container_self_item.get_name());
        assert_eq!('X', container_self_item.symbol.character);
        assert_eq!(Colour::White, container_self_item.symbol.colour);
        assert_eq!(1.0, container_self_item.weight);
        assert_eq!(1, container_self_item.value);

        for i in 1..=4 {
            let test_item = Item::with_defaults(format!("Test Item {}", i), 1.0, 100);
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
            tiles: Tiles { tiles : vec![
                vec![wall.clone(), wall.clone(), wall.clone()],
                vec![wall.clone(), rom.clone(), wall.clone()],
                vec![wall.clone(), wall.clone(), wall.clone()],
            ]},
            rooms: Vec::new(),
            containers: HashMap::new()
        };

        return Level { map: Some(map), characters: build_characters(Some(player), Vec::new())};
    }

    #[test]
    fn test_take_callback() {
        // GIVEN a valid level with an player inventory to extract items into
        let inventory = build(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1.0, 1,  ContainerType::OBJECT, 2);
        let _character_details = build_default_character_details();
        let player = Character::new(String::from("Test Player"), Position { x: 0, y: 0}, Symbol::new('@', Colour::Green), inventory);
        let mut level = build_test_level(player);
        let container_pos =  Position { x: 0, y: 0};

        // WHEN we call to handle a take callback with some of the items in a container
        let container = build_test_container();
        let mut selected_container_items = Vec::new();
        for i in 0..=1 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(2, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();

        let data = TakeItemsData { source: container.clone(), to_take: selected_container_items, position: Some(container_pos) };
        let view_result = ContainerFrameHandlerInputResult::TakeItems(data);
        let untaken = handle_callback(&mut level, container_pos, view_result).unwrap();

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
        let inventory = level.characters.get_player_mut().unwrap().get_inventory_mut();
        let updated_container_contents = inventory.get_contents();
        assert_eq!(2, updated_container_contents.len());
        assert_eq!(chosen_item_1, *updated_container_contents.get(0).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_container_contents.get(1).unwrap().get_self_item());
    }

    #[test]
    fn test_take_callback_too_many_items() {
        // GIVEN a valid map with an player inventory to extract items into
        let inventory = build(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1.0, 1,  ContainerType::OBJECT, 2);
        let _character_details = build_default_character_details();
        let player = Character::new(String::from("Test Player"), Position { x: 0, y: 0}, Symbol::new('@', Colour::Green), inventory);
        let mut level = build_test_level(player);
        let container_pos =  Position { x: 0, y: 0};

        // WHEN we call to handle a take callback with 3 items (with only space for 2 of them)
        let container = build_test_container();
        let _callback_container = container.clone();
        let mut selected_container_items = Vec::new();
        for i in 0..=2 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(3, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        let chosen_item_3 = selected_container_items.get(2).unwrap().clone();
        let data = TakeItemsData { source: container, to_take: selected_container_items, position: Some(container_pos) };
        let view_result = ContainerFrameHandlerInputResult::TakeItems(data);
        let untaken = handle_callback(&mut level, container_pos, view_result).unwrap();

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
        let inventory = level.characters.get_player_mut().unwrap().get_inventory_mut();
        let updated_container_contents = inventory.get_contents();
        assert_eq!(2, updated_container_contents.len());
        assert_eq!(chosen_item_1, *updated_container_contents.get(0).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_container_contents.get(1).unwrap().get_self_item());
    }

}