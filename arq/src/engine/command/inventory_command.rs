use std::io;
use std::io::{Error};


use termion::event::Key;
use crate::character::equipment::get_potential_slots;

use crate::engine::command::command::Command;
use crate::engine::container_util;
use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::objects::items::{Item};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;

use crate::view::util::callback::Callback;
use crate::view::character_info_view::{CharacterInfoView, CharacterInfoViewFrameHandler, TabChoice};
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, MoveItemsData, MoveToContainerChoiceData};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{DropItems, EquipItems, MoveItems, MoveToContainerChoice};

use crate::view::View;

const UI_USAGE_HINT: &str = "Up/Down - Move, Enter/q - Toggle/clear selection\nTab - Change tab, Esc - Exit";

pub struct InventoryCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

struct CallbackState<'a> {
    level : &'a mut Level,
    // When a container is needed i.e Dropping items
    container: Option<&'a mut Container>,
    data : ContainerFrameHandlerInputResult
}

fn equip_items(items: Vec<Item>, state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    // N.B cloned to avoid double borrow of characters, ensure re-assignment
    let mut equipment_snapshot = state.level.characters.get_player_mut().unwrap().get_equipment_mut().clone();
    if !items.is_empty() {
        let mut modified = Vec::new();
        {
            let inventory = state.level.characters.get_player_mut().unwrap().get_inventory_mut();
            for to_equip in items {
                let result = inventory.find_mut(&to_equip);
                if let Some(c) = result {
                    let c_copy = c.clone();
                    let result_item = c.get_self_item_mut();
                    // If the item is already equipped / un-equip it
                    if result_item.is_equipped() {
                        let equipped_slot = result_item.get_equipment_slot().unwrap();
                        equipment_snapshot.unequip(equipped_slot.clone());
                        result_item.unequip();
                        modified.push(result_item.clone());
                        log::info!("Un-equipped item {} from slot: {}", result_item.get_name(), equipped_slot);
                    } else {
                        // Otherwise try the potential slots for it
                        let potential_slots = get_potential_slots(result_item.item_type.clone());
                        let mut success = false;
                        let mut equip_result = Ok(());
                        let mut chosen_slot = None;
                        for slot in potential_slots {
                            equip_result = equipment_snapshot.equip(c_copy.clone(), slot.clone());
                            if equip_result.is_ok() {
                                success = true;
                                chosen_slot = Some(slot);
                                result_item.set_equipment_slot(chosen_slot.clone());
                                modified.push(result_item.clone());
                                log::info!("Equipped item {} to slot: {}", result_item.get_name(), chosen_slot.unwrap());
                                break;
                            } else if equip_result.is_err() {
                                log::info!("Failed to equip item {} with error: {}", result_item.get_name(), equip_result.unwrap_err());
                            } else {
                                log::info!("Failed to equip item {}", result_item.get_name());
                            }
                        }
                    }
                }
            }
        }

        // Ensure we persist the equipment changes
        // Return the callback with any updated item
        if !modified.is_empty() {
            state.level.characters.get_player_mut().unwrap().set_equipment(equipment_snapshot.clone());
        }
        return Some(EquipItems(modified));
    }

    None
}

fn drop_items(items: Vec<Item>, mut state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    let position = state.level.characters.get_player_mut().unwrap().get_global_position().clone();
    log::info!("InventoryCommand - Dropping {} items at position: {}, {}", items.len(),  position.x, position.y);

    // Find the container on the map and add the "container" wrappers there
    let mut undropped = Vec::new();
    for item in &items {
        undropped.push(item.clone());
    }

    if let Some(m) = state.level.get_map_mut() {
        if let Some(pos_container) = m.find_container_mut(position) {
            for item in &items {
                // Find the "container" wrappper matching the item returned
                if let Some(ref mut container) = state.container {
                    if let Some(container_item) = &mut container.find_mut(&item) {
                        let mut dropping_container_item = container_item.clone();
                        let self_item = dropping_container_item.get_self_item_mut();
                        self_item.unequip();
                        if pos_container.can_fit_container_item(&dropping_container_item) {
                            log::info!("Dropping item: {} into: {}", item.get_name(), pos_container.get_self_item().get_name());
                            pos_container.add(dropping_container_item);
                            let pos = undropped.iter().position(|x| x.id_equals(&item));
                            undropped.remove(pos.unwrap());
                        } else {
                            log::info!("Cannot fit item: {}  into: {}", item.get_name(), pos_container.get_self_item().get_name());
                        }
                    }
                }
            }
        }
    }
    return Some(DropItems(undropped));
}

fn build_container_choices<'a>(data: &'a MoveToContainerChoiceData, level: &'a mut Level) -> Result<ContainerFrameHandlerInputResult, Error> {
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
            log::info!("[inventory usage] Received data for DropItems with {} items", items.len());
            return drop_items(items.to_vec(), state);
        },
        MoveItems(data) => {
            log::info!("[inventory usage] Received data for MoveItems with {} items", data.to_move.len());
            return container_util::move_player_items(data, state.level);
        },
        EquipItems(ref data) => {
            log::info!("[inventory usage] Received data for EquipItems with {} items", data.len());
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
                log::info!("[inventory usage] Moving player items for MoveToContainerChoice...");
                return container_util::move_player_items(move_data, state.level);
            } else {
                // Build container choices and pass the result back down to the view/handlers
                log::info!("[inventory usage] Building choices for MoveToContainerChoice...");
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
        self.ui.set_console_buffer(UI_USAGE_HINT.to_string());

        let c = self.level.characters.get_player_mut().unwrap().get_inventory_mut();
        let mut callback_container: Container = c.clone();

        let frame_handler = CharacterInfoViewFrameHandler { tab_choice: TabChoice::INVENTORY, container_frame_handlers: Vec::new(), choice_frame_handler: None, character_view: None };

        let level = &mut self.level;
        let player = &mut level.characters.get_player_mut().unwrap().clone();
        let updated_inventory;
        {
            let mut character_info_view = CharacterInfoView { character: player, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, frame_handler, callback: Box::new(|_| {None}) };
            character_info_view.set_callback(Box::new(|data| {
                handle_callback(CallbackState { level, container: Some(&mut callback_container), data })
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
    use crate::character::builder::character_builder::{CharacterBuilder, CharacterPattern};
    use crate::character::characters::Characters;

    
    use crate::character::equipment::EquipmentSlot::PRIMARY;
    use crate::engine::command::inventory_command::{CallbackState, equip_items, handle_callback};
    use crate::engine::level::Level;
    
    
    use crate::map::objects::container::{Container, ContainerType};
    
    use crate::map::objects::items::{Item, ItemForm, MaterialType, Weapon};
    use crate::map::objects::weapon_builder::BladedWeaponType;
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::{Colour, TileType};
    use crate::map::Tiles;


    use crate::view::framehandler::container::{ContainerFrameHandlerInputResult};
    use crate::view::framehandler::container::ContainerFrameHandlerInputResult::EquipItems;
    

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container = Container::new(id, "Test Container".to_owned(), 'X', 1.0, 1, ContainerType::OBJECT, 100);
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

    fn build_test_level(area_container: Container) -> Level {
        let tile_library = crate::map::tile::build_library();
        let rom = tile_library[&TileType::Room].clone();
        let wall = tile_library[&TileType::Wall].clone();
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

        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .build(String::from("Test Player"));
        return  Level { map: Some(map) , characters: Characters::new( Some(player), Vec::new())  };
    }

    #[test]
    fn test_drop_callback() {
        // GIVEN a valid map with an area container to drop items into
        let area_container = Container::new(Uuid::new_v4(), "Floor".to_owned(), '$', 0.0, 0, ContainerType::AREA, 2);
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
        let undropped = handle_callback(CallbackState { level: &mut level, container: Some(&mut container), data: view_result }).unwrap();

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
        let area_container = Container::new(Uuid::new_v4(), "Floor".to_owned(), '$', 0.0, 0, ContainerType::AREA, 2);
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
        let undropped = handle_callback(CallbackState { level: &mut level, container: Some(&mut container), data: view_result }).unwrap();

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
        let area_container = Container::new(Uuid::new_v4(), "Floor".to_owned(), '$', 0.0, 0, ContainerType::AREA, 0);
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
        let undropped = handle_callback(CallbackState { level: &mut level, container: Some(&mut container), data: view_result }).unwrap();

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

    #[test]
    fn test_equip_callback() {
        // GIVEN a valid callback state
        let area_container = Container::new(Uuid::new_v4(), "Floor".to_owned(), '$', 0.0, 0, ContainerType::AREA, 2);
        let mut level = build_test_level(area_container);
        let player = level.characters.get_player_mut().unwrap();
        // AND the player has nothing equipped in the PRIMARY slot
        player.get_equipment_mut().unequip(PRIMARY);

        // AND an item that can be equipped that's inside the player's inventory
        let steel_sword = Item::weapon(Uuid::new_v4(), "".to_owned(), ItemForm::BLADED(BladedWeaponType::ARMING), MaterialType::STEEL, 'X', 3.0, 50, Weapon { damage: 10 });
        // AND this item should have no equipment slot set so far
        assert_eq!(None, steel_sword.get_equipment_slot());

        let to_equip = steel_sword.clone();
        let expected_id = to_equip.get_id().clone();
        player.get_inventory_mut().add_item(steel_sword);

        let data = ContainerFrameHandlerInputResult::EquipItems(vec![to_equip]);

        // WHEN we trigger a callback to equip that item (Steel Sword)
        let state = CallbackState { level: &mut level, container: None, data };
        let result = handle_callback(state).unwrap();

        match result {
            // THEN we expect a valid result with the item returned
            ContainerFrameHandlerInputResult::EquipItems(modified) => {
                assert_eq!(1, modified.len());
                let item1 = modified.get(0).unwrap();
                assert_eq!(expected_id, item1.get_id());
                assert_eq!("Steel Sword".to_owned(), item1.get_name());
                // AND the item should have the PRIMARY EquipmentSlot set against it
                assert_eq!(Some(PRIMARY), item1.get_equipment_slot());
            },
            _ => {
                assert!(false, "Expected a valid Some(EquipItems(m) to return!");
            }
        }
    }

    #[test]
    fn test_equip_items() {
        // GIVEN a valid callback state
        let area_container = Container::new(Uuid::new_v4(), "Floor".to_owned(), '$', 0.0, 0, ContainerType::AREA, 2);
        let mut level = build_test_level(area_container);
        let player = level.characters.get_player_mut().unwrap();
        // AND the player has nothing equipped in the PRIMARY slot
        player.get_equipment_mut().unequip(PRIMARY);

        // AND an item that can be equipped that's inside the player's inventory
        let steel_sword = Item::weapon(Uuid::new_v4(), "".to_owned(), ItemForm::BLADED(BladedWeaponType::ARMING), MaterialType::STEEL, 'X', 3.0, 50, Weapon { damage: 20 });
        // AND this item should have no equipment slot set so far
        assert_eq!(None, steel_sword.get_equipment_slot());

        let to_equip = steel_sword.clone();
        let expected_id = to_equip.get_id().clone();

        player.get_inventory_mut().add_item(steel_sword);

        // WHEN we call to equip that item (Steel Sword)
        let state = CallbackState { level: &mut level, container: None, data: ContainerFrameHandlerInputResult::None };
        let modified = equip_items(vec![to_equip], state);

        // THEN we expect a valid result with the item returned
        // AND the item should have the PRIMARY EquipmentSlot set against it
        if let Some(EquipItems(modified)) = modified {
            assert_eq!(1, modified.len());
            let item1 = modified.get(0).unwrap();
            assert_eq!(expected_id, item1.get_id());
            assert_eq!("Steel Sword".to_owned(), item1.get_name());
            assert_eq!(Some(PRIMARY), item1.get_equipment_slot());
        } else {
           assert!(false, "Expected a valid Some(EquipItems(m) to return!");
        }
    }

}