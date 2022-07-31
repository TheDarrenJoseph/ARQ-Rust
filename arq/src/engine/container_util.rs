use crate::character::Character;
use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, MoveItemsData, TakeItemsData};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{MoveItems, TakeItems};

pub fn take_items(data: TakeItemsData, level : &mut Level) -> Option<ContainerFrameHandlerInputResult> {
    // TODO have a find_player function?
    let player = &mut level.characters[0];
    log::info!("Found player: {}", player.get_name());
    if let Some(pos) = data.position {
        let mut taken = Vec::new();
        let mut untaken = Vec::new();
        for item in data.to_take {
            if let Some(container_item) = data.source.find(&item) {
                let inventory = player.get_inventory_mut();
                if inventory.can_fit_container_item(container_item) {
                    log::info!("Taking item: {}", item.get_name());
                    player.get_inventory_mut().add(container_item.clone());
                    taken.push(container_item.clone());
                } else {
                    untaken.push(item);
                }
            } else {
                untaken.push(item);
            }
        }
        if !taken.is_empty() || !untaken.is_empty() {
            let map_container = level.get_map_mut().unwrap().find_container(&data.source, pos);
            if let Some(source_container) = map_container {
                source_container.remove_matching_items(taken);
            }
        }
        log::info!("[take_items] returning TakeItems with {} un-taken items", untaken.len());
        let data = TakeItemsData { source: data.source.clone(), to_take: untaken, position: data.position };
        return Some(TakeItems(data));
    } else {
        log::error!("[take_items] No map position to take items from!");
    }
    return None
}

fn find_container_mut(root : &mut Container, target: Item) -> Option<&mut Container> {
    let mut target_result: Option<&mut Container> = None;
    let root_name = root.get_self_item().get_name().clone();
    if root.get_self_item().id_equals(&target) {
        return Some(root);
    } else if let Some(found) = root.find_mut(&target) {
        return Some(found);
    } else {
        log::error!("Couldn't find target container {} inside root {}.", target.get_name(), root_name);
        return None;
    }
}

fn move_to_container(root : &mut Container, source : Item, mut data: MoveItemsData) -> Option<ContainerFrameHandlerInputResult> {
    let from_container_name = source.get_name();
    let from_container_id = source.get_id();
    let target_result =  data.target_container.map_or_else(|| { None }, |t| { find_container_mut(root, t.get_self_item().clone()) });
        if let Some(target) = target_result {
            let mut moved = Vec::new();
            let mut unmoved = Vec::new();
            log::info!("Moving items from: {} ({}) into: {} ({})", from_container_name, from_container_id, target.get_self_item().get_name(), target.get_self_item().get_id());
            for item in data.to_move {
                if let Some(container_item) = data.source.find_mut(&item) {
                    if target.can_fit_container_item(container_item) {
                        target.add(container_item.clone());
                        moved.push(container_item.clone());
                    } else {
                        unmoved.push(item);
                    }
                }
            }
            let updated_target = Some(target.clone());

            if !moved.is_empty() || !unmoved.is_empty() {
                log::info!("Returning MoveItems response with {} moved, {} unmoved items", moved.len(), unmoved.len());
                if let Some(ref mut source_container) = find_container_mut(root, source) {
                    source_container.remove_matching_items(moved);
                    let data = MoveItemsData { source: source_container.clone(), to_move: unmoved, target_container: updated_target, position: data.position, target_item: None };
                    return Some(MoveItems(data));
                } else {
                    log::error!("Failed to move items. Failed to find source container.");
                }
            } else {
                log::error!("Failed to move items. {} moved, {} unmoved items", moved.len(), unmoved.len());
            }
        } else {
            log::error!("Failed to move items. Couldn't find target container in source container.");
            return None;
        }

    None
}

fn move_to_item_spot(source_container : &mut Container, mut data: MoveItemsData) -> Option<ContainerFrameHandlerInputResult> {
    if let Some(target_item) = data.target_item {
        if let Some(pos) = source_container.item_position(&target_item) {
            let mut unmoved = Vec::new();
            let mut moving = Vec::new();

            for item in &data.to_move {
                if let Some(container_item) = data.source.find_mut(&item) {
                    moving.push(container_item.clone());
                } else {
                    unmoved.push(item.clone());
                }
            }

            source_container.remove_matching_items(moving.clone());
            let target_pos = if pos >= moving.len() { pos - moving.len() } else { pos };
            source_container.insert(target_pos, moving.clone());
            let data = MoveItemsData { source: source_container.clone(), to_move: unmoved, target_container: None, target_item: Some(target_item.clone()), position: data.position };
            return Some(MoveItems(data));
        }
    }
    None
}

// Moves items between player inventory containers / into world container
pub fn move_player_items(data: MoveItemsData, level : &mut Level) -> Option<ContainerFrameHandlerInputResult> {
    if let Some(_) = data.position {
        // TODO potentially support this to allow moving into world containers
        None
    } else {
        let player : &mut Character = level.get_player_mut();
        let inventory : &mut Container = player.get_inventory_mut();
        let source;
        if inventory.id_equals(&data.source) {
            source = Some(inventory);
        } else {
            source = inventory.find_mut(data.source.get_self_item())
        }

        if let Some(s) = source {
            return if let Some(_) = data.target_container {
                let source_item = s.get_self_item().clone();
                log::info!("Attempting move to container..");
                return move_to_container(s, source_item, data);
            } else if let Some(_) = data.target_item {
                log::info!("Attempting move to item spot..");
                return move_to_item_spot(s, data);
            } else {
                None
            }
        } else {
            log::info!("[move_player_items] Failed to find source container!");
            None
        }
    }
}

// Moves items between world containers
pub fn move_items(data: MoveItemsData, level : &mut Level) -> Option<ContainerFrameHandlerInputResult> {
    return if let Some(_) = data.target_container {
        if let Some(pos) = data.position {
            if let Some(map) = &mut level.map {
                // Find the true instance of the source container on the map as our 'source_container'
                let map_container = map.find_container(&data.source, pos);
                if let Some(source_container) = map_container {
                    return move_to_container(source_container,data.source.get_self_item().clone(), data);
                }
            } else {
                log::error!("Cannot move items. No map provided");
            }
        } else {
            log::error!("Cannot move items. No map position provided");
        }
        log::error!("Failed to move items");
        None
    } else if let Some(ref target_item) = data.target_item {
        if let Some(pos) = data.position {
            if let Some(map) = &mut level.map {
                // Find the true instance of the source container on the map as our 'source_container'
                let map_container = map.find_container(&data.source, pos);
                if let Some(source_container) = map_container {
                    if let Some(_) = source_container.item_position(&target_item) {
                        return move_to_item_spot(source_container, data);
                    }
                }
            } else {
                log::error!("Cannot move items. No map provided");
            }
        } else {
            log::error!("Cannot move items. No map position provided");
        }
        log::error!("Failed to move items");
        None
    } else {
        log::error!("Cannot move items. No target provided.");
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::character::{build_character, build_default_character_details, build_player};
    use crate::engine::container_util::{move_items, move_player_items};
    use crate::engine::level::Level;
    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::objects::items::build_container_item;
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::{Colour, Tile};
    use crate::view::framehandler::container::ContainerFrameHandlerInputResult::MoveItems;
    use crate::view::framehandler::container::MoveItemsData;

    fn build_test_level(container_position: Position, area_container: Container) -> Level {
        let tile_library = crate::map::tile::build_library();
        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();
        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);

        let mut area_containers = HashMap::new();
        area_containers.insert(container_position.clone(), area_container);
        let map = crate::map::Map {
            area: map_area,
            tiles : vec![
                vec![ wall.clone(), wall.clone(), wall.clone() ],
                vec![ wall.clone(), rom.clone(), wall.clone() ],
                vec![ wall.clone(), wall.clone(), wall.clone() ],
            ],
            rooms: Vec::new(),
            containers: area_containers
        };

        let mut player = build_player(String::from("Test Player"), Position { x: 0, y: 0});
        return  Level { map: Some(map) , characters: vec![player] };
    }

    fn build_player_test_level() -> Level {
        let tile_library = crate::map::tile::build_library();
        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();
        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);
        let map = crate::map::Map {
            area: map_area,
            tiles : vec![
                vec![ wall.clone(), wall.clone(), wall.clone() ],
                vec![ wall.clone(), rom.clone(), wall.clone() ],
                vec![ wall.clone(), wall.clone(), wall.clone() ],
            ],
            rooms: Vec::new(),
            containers: HashMap::new()
        };
        let mut player = build_player(String::from("Test Player"), Position { x: 0, y: 0});
        return  Level { map: Some(map) , characters: vec![player] };
    }

    #[test]
    fn test_move_items_into_container() {
        // GIVEN a valid map
        // that holds a source container containing 3 containers
        let mut source_container =  build(Uuid::new_v4(), "Source Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let to_move = vec![container1.get_self_item().clone()];
        source_container.push(vec![container1, container2, container3]);
        assert_eq!(3, source_container.get_item_count());

        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let target = source_container.get(2).clone();
        let target_item = target.get_self_item().clone();
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to move container 1 into container 3
        let data = MoveItemsData { source, to_move, target_container: Some(target), target_item: None, position: Some(container_pos) };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect a valid result
        if let Some(input_result) = result {
            match input_result {
                MoveItems(result_data) => {
                    // AND the source/targets should be returned with no outstanding to_move data
                    assert!(data_expected.source.id_equals(&result_data.source));
                    assert!(data_expected.target_container.unwrap().id_equals(&result_data.target_container.unwrap()));
                    assert!(result_data.to_move.is_empty());

                    // AND The map 'source' container will have the items removed
                    let mut map_container = level.get_map_mut().unwrap().find_container(&data_expected.source, container_pos);
                    if let Some(c) = map_container {
                        assert_eq!(2, c.get_item_count());
                        // AND The 'target' container will contain the new items
                        if let Some(container_item) = c.find(&target_item) {
                            assert_eq!(1, container_item.get_item_count());
                        }
                        return; // pass
                    }
                },
                _ => {}
            }
        }
        assert!(false);
    }

    #[test]
    fn test_move_items_bottom() {
        // GIVEN a valid map
        // that holds a source container containing 6 containers (Each with a unique name)
        let mut source_container =  build(Uuid::new_v4(), "Source Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container4 =  build(Uuid::new_v4(), "Test Container 4".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container5 =  build(Uuid::new_v4(), "Test Container 5".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container6 =  build(Uuid::new_v4(), "Test Container 6".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        // Clone everything before moving
        let to_move = vec![container1.get_self_item().clone(), container2.get_self_item().clone()];
        source_container.push(vec![container1, container2, container3,  container4,  container5, container6], );
        let source_copy = source_container.clone();
        assert_eq!(6, source_container.get_item_count());

        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let target_item = source_container.get(5).get_self_item().clone();
        let expected_target = target_item.clone();
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to move container 1 and 2 to the bottom of the list (Container 6's location)
        let data = MoveItemsData { source, to_move, target_container: None, target_item: Some(target_item), position: Some(container_pos) };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect a valid result
        if let Some(input_result) = result {
            match input_result {
                MoveItems(result_data) => {
                    // AND the source/targets should be returned with no outstanding to_move data
                    assert!(data_expected.source.id_equals(&result_data.source));
                    assert_eq!(data_expected.target_item.unwrap().get_id(), result_data.target_item.unwrap().get_id());
                    assert_eq!(0, result_data.to_move.len());

                    // AND The map 'source' container will have it's items reshuffled
                    let mut map_container = level.get_map_mut().unwrap().find_container(&data_expected.source, container_pos);
                    if let Some(c) = map_container {
                        assert_eq!(6, c.get_item_count());
                        let contents = c.get_contents();
                        assert_eq!(source_copy.get(2).get_self_item().get_name(), contents[0].get_self_item().get_name());
                        assert_eq!(source_copy.get(3).get_self_item().get_name(), contents[1].get_self_item().get_name());
                        assert_eq!(source_copy.get(4).get_self_item().get_name(), contents[2].get_self_item().get_name());
                        assert_eq!(source_copy.get(0).get_self_item().get_name(), contents[3].get_self_item().get_name());
                        assert_eq!(source_copy.get(1).get_self_item().get_name(), contents[4].get_self_item().get_name());
                        assert_eq!(source_copy.get(5).get_self_item().get_name(), contents[5].get_self_item().get_name());
                        return; // pass
                    }
                },
                _ => {}
            }
        }
        assert!(false);

    }

    #[test]
    fn test_move_items_top() {
        // GIVEN a valid map
        // that holds a source container containing 6 containers (Each with a unique name)
        let mut source_container =  build(Uuid::new_v4(), "Source Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container4 =  build(Uuid::new_v4(), "Test Container 4".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container5 =  build(Uuid::new_v4(), "Test Container 5".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container6 =  build(Uuid::new_v4(), "Test Container 6".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        // Clone everything before moving
        let to_move = vec![container5.get_self_item().clone(), container6.get_self_item().clone()];
        source_container.push(vec![container1, container2, container3,  container4,  container5, container6], );
        let source_copy = source_container.clone();
        assert_eq!(6, source_container.get_item_count());

        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let target_item = source_container.get(0).get_self_item().clone();
        let expected_target = target_item.clone();
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to move container 5 and 6 to the top of the list (Container 1's location)
        let data = MoveItemsData { source, to_move, target_container: None, target_item: Some(target_item), position: Some(container_pos) };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect a valid result
        if let Some(input_result) = result {
            match input_result {
                MoveItems(result_data) => {
                    // AND the source/targets should be returned with no outstanding to_move data
                    assert!(data_expected.source.id_equals(&result_data.source));
                    assert_eq!(data_expected.target_item.unwrap().get_id(), result_data.target_item.unwrap().get_id());
                    assert_eq!(0, result_data.to_move.len());

                    // AND The map 'source' container will have it's items reshuffled
                    let mut map_container = level.get_map_mut().unwrap().find_container(&data_expected.source, container_pos);
                    if let Some(c) = map_container {
                        assert_eq!(6, c.get_item_count());
                        let contents = c.get_contents();
                        assert_eq!(source_copy.get(4).get_self_item().get_name(), contents[0].get_self_item().get_name());
                        assert_eq!(source_copy.get(5).get_self_item().get_name(), contents[1].get_self_item().get_name());
                        assert_eq!(source_copy.get(0).get_self_item().get_name(), contents[2].get_self_item().get_name());
                        assert_eq!(source_copy.get(1).get_self_item().get_name(), contents[3].get_self_item().get_name());
                        assert_eq!(source_copy.get(2).get_self_item().get_name(), contents[4].get_self_item().get_name());
                        assert_eq!(source_copy.get(3).get_self_item().get_name(), contents[5].get_self_item().get_name());
                        return; // pass
                    }
                },
                _ => {}
            }
        }
        assert!(false);
    }

    #[test]
    fn test_move_item_middle() {
        // GIVEN a valid map
        // that holds a source container containing 6 containers (Each with a unique name)
        let mut source_container =  build(Uuid::new_v4(), "Source Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container4 =  build(Uuid::new_v4(), "Test Container 4".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container5 =  build(Uuid::new_v4(), "Test Container 5".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container6 =  build(Uuid::new_v4(), "Test Container 6".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        // Clone everything before moving
        let to_move = vec![container1.get_self_item().clone(), container2.get_self_item().clone()];
        source_container.push(vec![container1, container2, container3,  container4,  container5, container6], );
        let source_copy = source_container.clone();
        assert_eq!(6, source_container.get_item_count());

        // WHEN we call to move container 1 and 2 to the middle of the list (Container 5's location)
        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let target_item = source_container.get(4).get_self_item().clone();
        let expected_target = target_item.clone();
        let mut level = build_test_level(container_pos, source_container);
        let data = MoveItemsData { source, to_move, target_container: None, target_item: Some(target_item), position: Some(container_pos) };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect a valid result
        if let Some(input_result) = result {
            match input_result {
                MoveItems(result_data) => {
                    // AND the source/targets should be returned with no outstanding to_move data
                    assert!(data_expected.source.id_equals(&result_data.source));
                    assert_eq!(data_expected.target_item.unwrap().get_id(), result_data.target_item.unwrap().get_id());
                    assert_eq!(0, result_data.to_move.len());

                    // AND The map 'source' container will have it's items reshuffled
                    let mut map_container = level.get_map_mut().unwrap().find_container(&data_expected.source, container_pos);
                    if let Some(c) = map_container {
                        assert_eq!(6, c.get_item_count());
                        let contents = c.get_contents();
                        assert_eq!(source_copy.get(2).get_self_item().get_name(), contents[0].get_self_item().get_name());
                        assert_eq!(source_copy.get(3).get_self_item().get_name(), contents[1].get_self_item().get_name());
                        assert_eq!(source_copy.get(0).get_self_item().get_name(), contents[2].get_self_item().get_name());
                        assert_eq!(source_copy.get(1).get_self_item().get_name(), contents[3].get_self_item().get_name());
                        assert_eq!(source_copy.get(4).get_self_item().get_name(), contents[4].get_self_item().get_name());
                        assert_eq!(source_copy.get(5).get_self_item().get_name(), contents[5].get_self_item().get_name());
                        return; // pass
                    }
                },
                _ => {}
            }
        }
        assert!(false);
    }

    fn test_move_split_items() {
        // GIVEN a valid map
        // that holds a source container containing 6 containers (Each with a unique name)
        let mut source_container =  build(Uuid::new_v4(), "Source Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container4 =  build(Uuid::new_v4(), "Test Container 4".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container5 =  build(Uuid::new_v4(), "Test Container 5".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container6 =  build(Uuid::new_v4(), "Test Container 6".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        // Clone everything before moving
        let to_move = vec![container1.get_self_item().clone(), container6.get_self_item().clone()];
        source_container.push(vec![container1, container2, container3,  container4,  container5, container6], );
        let source_copy = source_container.clone();
        assert_eq!(6, source_container.get_item_count());

        // WHEN we call to move container 1 and 6 to container 2's location (index 1)
        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let target_item = source_container.get(1).get_self_item().clone();
        let expected_target = target_item.clone();
        let mut level = build_test_level(container_pos, source_container);
        let data = MoveItemsData { source, to_move, target_container: None, target_item: Some(target_item), position: Some(container_pos) };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect a valid result
        if let Some(input_result) = result {
            match input_result {
                MoveItems(result_data) => {
                    // AND the source/targets should be returned with no outstanding to_move data
                    assert!(data_expected.source.id_equals(&result_data.source));
                    assert_eq!(data_expected.target_item.unwrap().get_id(), result_data.target_item.unwrap().get_id());
                    assert_eq!(0, result_data.to_move.len());

                    // AND The map 'source' container will have it's items reshuffled
                    let mut map_container = level.get_map_mut().unwrap().find_container(&data_expected.source, container_pos);
                    if let Some(c) = map_container {
                        assert_eq!(6, c.get_item_count());
                        let contents = c.get_contents();
                        assert_eq!(source_copy.get(0).get_self_item().get_name(), contents[0].get_self_item().get_name());
                        assert_eq!(source_copy.get(5).get_self_item().get_name(), contents[1].get_self_item().get_name());
                        assert_eq!(source_copy.get(1).get_self_item().get_name(), contents[2].get_self_item().get_name());
                        assert_eq!(source_copy.get(2).get_self_item().get_name(), contents[3].get_self_item().get_name());
                        assert_eq!(source_copy.get(3).get_self_item().get_name(), contents[4].get_self_item().get_name());
                        assert_eq!(source_copy.get(4).get_self_item().get_name(), contents[5].get_self_item().get_name());
                        return; // pass
                    }
                },
                _ => {}
            }
        }
        assert!(false)
    }

    #[test]
    fn test_move_items_no_position() {
        // GIVEN a valid map
        // that holds a source container containing 3 containers
        let mut source_container =  build(Uuid::new_v4(), "Source Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let to_move = vec![container1.get_self_item().clone()];
        source_container.push(vec![container1, container2, container3]);
        assert_eq!(3, source_container.get_item_count());

        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let target = source_container.get(2).clone();
        let target_item = target.get_self_item().clone();
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to move container 1 into container 3 without a position for the container
        let data = MoveItemsData { source, to_move, target_container: Some(target), target_item: None, position: None };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect None to return
        assert!(result.is_none());
    }

    #[test]
    fn test_move_player_items_from_lower_to_root() {
        // GIVEN a player inventory containing a nested container (Bag)
        let mut inventory =  build(Uuid::new_v4(), "Player Inventory".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let mut bag =  build(Uuid::new_v4(), "Bag".to_owned(), 'X', 5, 1, ContainerType::OBJECT, 100);

        // AND both the parent container and bag contains some other items
        let item1 =  build(Uuid::new_v4(), "Test Item 1".to_owned(), 'X', 1, 1,  ContainerType::ITEM, 0);
        let item2 =  build(Uuid::new_v4(), "Test Item 2".to_owned(), 'X', 1, 1,  ContainerType::ITEM, 0);
        let item3 =  build(Uuid::new_v4(), "Test Item 3".to_owned(), 'X', 1, 1,  ContainerType::ITEM, 0);

        let item4 =  build(Uuid::new_v4(), "Test Item 4".to_owned(), 'X', 1, 1,  ContainerType::ITEM, 0);
        let item5 =  build(Uuid::new_v4(), "Test Item 5".to_owned(), 'X', 1, 1,  ContainerType::ITEM, 0);
        let item6 =  build(Uuid::new_v4(), "Test Item 6".to_owned(), 'X', 1, 1,  ContainerType::ITEM, 0);
        let to_move = vec![item6.get_self_item().clone()];

        // AND we're moving items from the underlying bag into the main inventory (root)
        let target = inventory.clone();

        let source = bag.clone();
        let bag_item = bag.get_self_item().clone();

        bag.push(vec![item4, item5, item6]);
        inventory.push(vec![item1, item2, item3, bag]);
        // 7 total contents (including the Bag contents)
        assert_eq!(7, inventory.get_item_count());
        // Root container has items 1-3 and the bag at the top level
        assert_eq!(4, inventory.get_content_count());

        let mut level = build_player_test_level();
        level.get_player_mut().set_inventory(inventory.clone());

        // WHEN we try to move an item from the bag into the root container
        let data = MoveItemsData { source, to_move, target_container: Some(target), target_item: None, position: None };
        let result = move_player_items(data, &mut level);

        // THEN we expect a result to return
        assert!(result.is_some());

        if let Some(MoveItems(d)) = result {
            // with 0 unmoved items
            assert_eq!(0, d.to_move.len());
        } else {
            assert!(false, "Unexpected data type returned");
        }

        let updated_inventory = level.get_player_mut().get_inventory_mut();
        // AND the player's inventory should not have 5 items in it's content count
        assert_eq!(5, updated_inventory.get_content_count());
        // AND The bag should have only 2 items now
        if let Some(b) = updated_inventory.find(&bag_item) {
            assert_eq!(3, b.get_content_count());
        } else {
            assert!(false, "Couldn't find Bag in the updated inventory!");
        }

    }
}