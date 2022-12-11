use std::io;
use std::io::{Error};

use termion::event::Key;

use crate::engine::command::command::Command;
use crate::engine::container_util;
use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;

use crate::view::callback::Callback;
use crate::view::character_info::{CharacterInfoView, CharacterInfoViewFrameHandler, TabChoice};
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, MoveItemsData, MoveToContainerChoiceData};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{DropItems, EquipItems, MoveItems, MoveToContainerChoice};

use crate::view::View;

pub struct InventoryCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

struct CallbackState<'a> {
    pub level : &'a mut Level,
    pub container: &'a mut Container,
    pub data : ContainerFrameHandlerInputResult
}

fn equip_items(items: Vec<Item>, state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    let inventory = state.level.characters.get_player_mut().unwrap().get_inventory_mut();
    if !items.is_empty() {
        let mut toggled = Vec::new();
        for to_equip in items {
            let result = inventory.find_mut(&to_equip);
            if let Some(c) = result {
                if c.get_self_item_mut().toggle_equipped() {
                    toggled.push(c.get_self_item().clone());
                }
            }
        }
        return Some(EquipItems(toggled));
    }

    None
}

fn drop_items(items: Vec<Item>, state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    let position = state.level.characters.get_player_mut().unwrap().get_position().clone();
    log::info!("InventoryCommand - Dropping {} items at position: {}, {}", items.len(),  position.x, position.y);

    // Find the container on the map and add the "container" wrappers there
    let mut undropped = Vec::new();
    if let Some(m) = state.level.get_map_mut() {
        if let Some(pos_container) = m.find_container_mut(position) {
            for item in items {
                // Find the "container" wrappper matching the item returned
                if let Some(container_item) = state.container.find_mut(&item) {
                    let mut dropping_container_item = container_item.clone();
                    let self_item = dropping_container_item.get_self_item_mut();
                    self_item.unequip();
                    if pos_container.can_fit_container_item(&dropping_container_item) {
                        log::info!("Dropping item: {} into: {}", item.get_name(), pos_container.get_self_item().get_name());
                        pos_container.add(dropping_container_item)
                    } else {
                        log::info!("Cannot fit item: {}  into: {}", item.get_name(), pos_container.get_self_item().get_name());
                        undropped.push(item);
                    }
                }
            }
        }
    }
    return Some(DropItems(undropped));
}

fn build_container_choices(data: &MoveToContainerChoiceData, level: &mut Level) -> Result<ContainerFrameHandlerInputResult, Error> {
    let inventory = level.characters.get_player_mut().unwrap().get_inventory_mut();
    let sub_containers_result = container_util::build_container_choices(&data.source, inventory);

    let sub_containers = sub_containers_result.unwrap();
    let mut result_data = data.clone();
    result_data.choices = sub_containers;
    return Ok(MoveToContainerChoice(result_data));
}

fn handle_callback(state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    match state.data {
        DropItems(ref items) => {
            log::info!("[inventory command] Received data for DropItems with {} items", items.len());
            return drop_items(items.to_vec(), state);
        },
        MoveItems(data) => {
            log::info!("[inventory command] Received data for MoveItems with {} items", data.to_move.len());
            return container_util::move_player_items(data, state.level);
        },
        EquipItems(ref data) => {
            log::info!("[inventory command] Received data for EquipItems with {} items", data.len());
            return equip_items(data.clone(), state);
        },
        MoveToContainerChoice(ref data) => {
            return if let Some(_target) = &data.target_container {
                // Translate to the typical moving data
                let move_data = MoveItemsData {
                    source: data.source.clone(),
                    to_move: data.to_move.clone(),
                    target_container: data.target_container.clone(),
                    target_item: None,
                    position: None
                };
                log::info!("[inventory command] Moving player items for MoveToContainerChoice...");
                return container_util::move_player_items(move_data, state.level);
            } else {
                // Build container choices and pass the result back down to the view/handlers
                log::info!("[inventory command] Building choices for MoveToContainerChoice...");
                build_container_choices(data, state.level).ok()
            }
        }
        _ => {
            return None
        }
    }
}

impl <B: tui::backend::Backend> InventoryCommand<'_, B> {

    fn open_inventory(&mut self) -> Result<(), io::Error> {
        log::info!("Player opening inventory.");
        let c = self.level.characters.get_player_mut().unwrap().get_inventory_mut();
        let mut callback_container: Container = c.clone();

        let frame_handler = CharacterInfoViewFrameHandler { tab_choice: TabChoice::INVENTORY, container_frame_handlers: Vec::new(), choice_frame_handler: None, character_view: None };

        self.ui.console_print("Up/Down - Move\nEnter - Toggle selection".to_string());

        let level = &mut self.level;
        let player = &mut level.characters.get_player_mut().unwrap().clone();
        let updated_inventory;
        {
            let mut character_info_view = CharacterInfoView { character: player, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, frame_handler, callback: Box::new(|_| {None}) };
            character_info_view.set_callback(Box::new(|data| {
                handle_callback(CallbackState { level, container: &mut callback_container, data })
            }));
            match character_info_view.begin() {
                Ok(_) => {
                    updated_inventory = character_info_view.frame_handler.container_frame_handlers.get(0).unwrap().container.clone();
                },
                Err(e) => {
                    return Err(e)
                }
            }
        }
        level.characters.get_player_mut().unwrap().set_inventory(updated_inventory);
        return Ok(())
    }
}

impl <B: tui::backend::Backend> Command for InventoryCommand<'_, B> {
    fn handles_key(&self, key: Key) -> bool {
        return match key {
            Key::Char('i') => {
                true
            },
            _ => {
                false
            }
        };
    }

    fn handle(&mut self, _: Key) -> Result<(), io::Error> {
        return self.open_inventory();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    

    
    
    
    
    
    
    use uuid::Uuid;

    use crate::character::build_player;
    use crate::characters::build_characters;
    use crate::engine::command::inventory_command::{CallbackState, handle_callback};
    use crate::engine::level::Level;
    
    use crate::map::objects::container;
    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::objects::items;
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::{Colour, Tile};
    use crate::map::Tiles;


    use crate::view::framehandler::container::{ContainerFrameHandlerInputResult};

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container =  build(id, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
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

    fn build_test_level(area_container: Container) -> Level {
        let tile_library = crate::map::tile::build_library();
        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();
        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);

        assert_eq!(0, area_container.get_contents().len());
        let mut area_containers = HashMap::new();
        area_containers.insert(map_pos.clone(), area_container);
        let map = crate::map::Map {
            area: map_area,
            tiles : Tiles { tiles : vec![
                vec![ wall.clone(), wall.clone(), wall.clone() ],
                vec![ wall.clone(), rom.clone(), wall.clone() ],
                vec![ wall.clone(), wall.clone(), wall.clone() ],
            ]},
            rooms: Vec::new(),
            containers: area_containers
        };

        let player = build_player(String::from("Test Player"), Position { x: 0, y: 0});
        return  Level { map: Some(map) , characters: build_characters( Some(player), Vec::new())  };
    }

    #[test]
    fn test_drop_callback() {
        // GIVEN a valid map with an area container to drop items into
        let area_container = container::build(Uuid::new_v4(), "Floor".to_owned(), '$', 0, 0,  ContainerType::AREA, 2);
        let mut level = build_test_level(area_container);

        // WHEN we call to handle a drop callback with some of the items in the container
        let mut container = build_test_container();
        let mut selected_container_items = Vec::new();
        for i in 0..=1 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(2, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        let view_result = ContainerFrameHandlerInputResult::DropItems(selected_container_items);
        let undropped = handle_callback(CallbackState { level: &mut level, container: &mut container, data: view_result }).unwrap();

        // THEN we expect a DropItems returned with 0 un-dropped items
        match undropped {
            ContainerFrameHandlerInputResult::DropItems(u) => {
                assert_eq!(0, u.len());
            },
            _ => {
                assert!(false);
            }
        }

        // AND we expect the area container to contain the 2 items dropped
        let updated_container = level.map.unwrap().get_container(Position { x: 0, y: 0 }).unwrap().clone();
        let updated_container_contents = updated_container.get_contents();
        assert_eq!(2,updated_container_contents.len());
        assert_eq!(chosen_item_1, *updated_container_contents.get(0).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_container_contents.get(1).unwrap().get_self_item());
    }

    #[test]
    fn test_drop_callback_too_many_items() {
        // GIVEN a valid map with an area container to drop items into
        let area_container = container::build(Uuid::new_v4(), "Floor".to_owned(), '$', 0, 0,  ContainerType::AREA, 2);
        let mut level = build_test_level(area_container);

        // WHEN we call to handle a drop callback with too many items to fit in the current area container (3 with space for 2)
        let mut container = build_test_container();
        let mut selected_container_items = Vec::new();
        for i in 0..=2 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(3, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        let chosen_item_3 = selected_container_items.get(2).unwrap().clone();
        let view_result = ContainerFrameHandlerInputResult::DropItems(selected_container_items);
        let undropped = handle_callback(CallbackState { level: &mut level, container: &mut container, data: view_result }).unwrap();

        // THEN we expect a DropItems returned with 1 un-dropped items
        match undropped {
            ContainerFrameHandlerInputResult::DropItems(u) => {
                assert_eq!(1, u.len());
                assert_eq!(chosen_item_3, *u.get(0).unwrap());
            },
            _ => {
                assert!(false);
            }
        }

        // AND we expect the area container to contain the first 2 items dropped
        let updated_container = level.map.unwrap().get_container(Position { x: 0, y: 0 }).unwrap().clone();
        let updated_container_contents = updated_container.get_contents();
        assert_eq!(2,updated_container_contents.len());
        assert_eq!(chosen_item_1, *updated_container_contents.get(0).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_container_contents.get(1).unwrap().get_self_item());
    }

    #[test]
    fn test_drop_callback_zero_weightlimit() {
        // GIVEN a valid map with an area container to drop items into (with a zero weightlimit)
        let area_container = container::build(Uuid::new_v4(), "Floor".to_owned(), '$', 0, 0,  ContainerType::AREA, 0);
        let mut level = build_test_level(area_container);

        // WHEN we call to handle a drop callback
        let mut container = build_test_container();
        let mut selected_container_items = Vec::new();
        for i in 0..=1 {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(2, selected_container_items.len());
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        let view_result = ContainerFrameHandlerInputResult::DropItems(selected_container_items);
        let undropped = handle_callback(CallbackState { level: &mut level, container: &mut container, data: view_result }).unwrap();

        // THEN we expect a DropItems returned with 2 un-dropped items
        match undropped {
            ContainerFrameHandlerInputResult::DropItems(u) => {
                assert_eq!(2, u.len());
                assert_eq!(chosen_item_1, *u.get(0).unwrap());
                assert_eq!(chosen_item_2, *u.get(1).unwrap());
            },
            _ => {
                assert!(false);
            }
        }

        // AND we expect the area container to contain nothing
        let updated_container = level.map.unwrap().get_container(Position { x: 0, y: 0 }).unwrap().clone();
        let updated_container_contents = updated_container.get_contents();
        assert_eq!(0,updated_container_contents.len());
    }

}