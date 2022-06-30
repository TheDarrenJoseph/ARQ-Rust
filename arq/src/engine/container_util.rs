use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::map::position::Position;
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
                let inventory = player.get_inventory();
                if inventory.can_fit_container_item(container_item) {
                    log::info!("Taking item: {}", item.get_name());
                    player.get_inventory().add(container_item.clone());
                    taken.push(container_item.clone());
                } else {
                    untaken.push(item);
                }
            } else {
                untaken.push(item);
            }
        }
        if !taken.is_empty() || !untaken.is_empty() {
            let mut map_container = level.get_map_mut().unwrap().find_container(&data.source, pos);
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

fn move_to_container(source : &mut Container, mut data: MoveItemsData) -> Option<ContainerFrameHandlerInputResult> {
    let from_container_name = source.get_self_item().get_name();
    let from_container_id = source.get_self_item().get_id();

    if let Some(target_container) = data.target_container {
        if let Some(target) = source.find_mut(target_container.get_self_item()) {
            let mut moved = Vec::new();
            let mut unmoved = Vec::new();
            let mut updated_target = None;
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
            updated_target = Some(target.clone());

            if !moved.is_empty() || !unmoved.is_empty() {
                log::info!("Returning MoveItems response with {} moved, {} unmoved items", moved.len(), unmoved.len());
                source.remove_matching_items(moved);

                let data = MoveItemsData { source: source.clone(), to_move: unmoved, target_container: updated_target, position: data.position, target_item: None };
                return Some(MoveItems(data));
            }
        }
    }
    None
}

// TODO Moves items between player inventory containers / into world container
pub fn move_player_items(mut data: MoveItemsData, level : &mut Level) -> Option<ContainerFrameHandlerInputResult> {
    if let Some(pos) = data.position {
        // TODO potentially support this to allow moving into world containers
        None
    } else {
        return if let Some(ref target_container) = data.target_container {
            let mut player = level.get_player_mut();
            let mut inventory = player.get_inventory();
            let mut source = None;

            if data.source.id_equals(inventory) {
                source = Some(inventory)
            } else {
                source = inventory.find_mut(data.source.get_self_item())
            }

            if let Some(s) = source {
                log::info!("Returning MoveItems response");
                return move_to_container(s, data)
            }
            None
        } else if let Some(target_item) = data.target_item {
            //TODO
            None
        } else {
            None
        }
    }
}

// Moves items between world containers
pub fn move_items(mut data: MoveItemsData, level : &mut Level) -> Option<ContainerFrameHandlerInputResult> {
    return if let Some(ref mut target_container) = data.target_container {
        if let Some(pos) = data.position {
            if let Some(map) = &mut level.map {
                // Find the true instance of the source container on the map as our 'source_container'
                let mut map_container = map.find_container(&data.source, pos);
                if let Some(source_container) = map_container {
                    return move_to_container(source_container, data);
                }
            } else {
                log::error!("Cannot move items. No map provided");
            }
        } else {
            log::error!("Cannot move items. No map position provided");
        }
        log::error!("Failed to move items");
        None
    } else if let Some(target_item) = data.target_item {
        if let Some(pos) = data.position {
            if let Some(map) = &mut level.map {
                // Find the true instance of the source container on the map as our 'source_container'
                let mut map_container = map.find_container(&data.source, pos);
                if let Some(source_container) = map_container {
                    let from_container_name = source_container.get_self_item().get_name();
                    let from_container_id = source_container.get_self_item().get_id();
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
    use crate::engine::container_util::move_items;
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
}