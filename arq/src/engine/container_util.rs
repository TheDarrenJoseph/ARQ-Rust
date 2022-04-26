use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::map::position::Position;
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, MoveItemsData, TakeItemsData};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{MOVE_ITEMS, TAKE_ITEMS};

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
        log::info!("[take_items] returning TAKE_ITEMS with {} un-taken items", untaken.len());
        let data = TakeItemsData { source: data.source.clone(), to_take: untaken, position: data.position };
        return Some(TAKE_ITEMS(data));
    } else {
        log::error!("[take_items] No map position to take items from!");
    }
    return None
}

// TODO Moves items between player inventory containers / into world container

// Moves items between world containers
pub fn move_items(mut data: MoveItemsData, level : &mut Level) -> Option<ContainerFrameHandlerInputResult> {
    return if let Some(ref mut target) = data.target {
        if let Some(pos) = data.position {
            if let Some(map) = &mut level.map {
                // Find the true instance of the source container on the map as our 'source_container'
                let mut map_container = map.find_container(&data.source, pos);
                if let Some(source_container) = map_container {
                    let from_container_name = source_container.get_self_item().get_name();
                    let from_container_id = source_container.get_self_item().get_id();
                    let mut moved = Vec::new();
                    let mut unmoved = Vec::new();
                    let mut updated_target = None;
                    // Find the true instance of the target container
                    if let Some(target) = source_container.find_mut(target.get_self_item()) {
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
                    }
                    if !moved.is_empty() || !unmoved.is_empty() {
                        log::info!("Returning MOVE_ITEMS response with {} unmoved items", unmoved.len());
                        source_container.remove_matching_items(moved);

                        let data = MoveItemsData { source: source_container.clone(), to_move: unmoved, target: updated_target, position: data.position };
                        return Some(MOVE_ITEMS(data));
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
        // TODO Index based move for when this is not a container target?
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
    use crate::view::framehandler::container::ContainerFrameHandlerInputResult::MOVE_ITEMS;
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
        let data = MoveItemsData { source, to_move, target: Some(target), position: Some(container_pos) };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect a valid result
        if let Some(input_result) = result {
            match input_result {
                MOVE_ITEMS(result_data) => {
                    // AND the source/targets should be returned with no outstanding to_move data
                    assert!(data_expected.source.id_equals(&result_data.source));
                    assert!(data_expected.target.unwrap().id_equals(&result_data.target.unwrap()));
                    assert!(result_data.to_move.is_empty());

                    // AND The map 'source' container will have the items removed
                    let mut map_container = level.get_map_mut().unwrap().find_container(&data_expected.source, container_pos);
                    if let Some(c) = map_container {
                        assert_eq!(2, c.get_item_count());
                        // AND The 'target' container will contain the new items
                        if let Some(container_item) = c.find(&target_item) {
                            assert_eq!(1, container_item.get_item_count());
                        }
                        return;
                    }
                },
                _ => {}
            }
        }
        assert!(false);
    }

    #[test]
    fn test_move_items_bottom() {
        assert!(false)
        /*
        // GIVEN a valid map with an player inventory to extract items into
        let inventory = build(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 2);
        let character_details = build_default_character_details();
        let player = build_character(String::from("Test Player") , Position { x: 0, y: 0}, inventory);
        let mut level = build_test_level(player);
        let container_pos =  Position { x: 0, y: 0};

        // GIVEN a valid view
        let mut container = build_test_container();
        let mut view : ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;
        assert_eq!(0, view.item_list_selection.get_true_index());
        let mut contents = view.container.get_contents();
        assert_eq!(4, contents.len());
        // with a series of items
        assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[3].get_self_item().get_name());

        // AND we've started selecting items
        view.toggle_select();
        // AND we've selected the first 2 items
        view.move_down();
        view.toggle_select();

        // WHEN we move to the bottom of the view and try to move the items
        view.page_down();
        view.move_selection();

        // THEN we expect the focused index to remain at the top of the view
        assert_eq!(0, view.item_list_selection.get_true_index());

        // AND the chosen items will be moved to the bottom of the view above the last item
        let contents = view.container.get_contents();
        assert_eq!(4, contents.len());
        assert_eq!("Test Item 3", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 1", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[3].get_self_item().get_name());
        */
    }


    #[test]
    fn test_move_items_top() {
        assert!(false)
        /*
    // GIVEN a valid view
    let mut container = build_test_container();
    let mut view : ContainerFrameHandler = build_default_container_view(container);
    view.item_list_selection.page_line_count = 4;
    assert_eq!(0, view.item_list_selection.get_true_index());
    let mut contents = view.container.get_contents();
    assert_eq!(4, contents.len());
    // with a series of items
    assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
    assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[3].get_self_item().get_name());

    // AND we've started selecting items at index 2
    view.move_down();
    view.move_down();
    view.toggle_select();
    // AND we've selected the last 2 items
    view.move_down();
    view.toggle_select();

    // WHEN we move to the top of the view and try to move the items
    view.page_up();
    view.move_selection();

    // THEN we expect the focused index to remain at the top of the view
    assert_eq!(0, view.item_list_selection.get_true_index());

    // AND the chosen items will be moved to the top of the view
    let contents = view.container.get_contents();
    assert_eq!(4, contents.len());
    assert_eq!("Test Item 3", contents[0].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 1", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 2", contents[3].get_self_item().get_name());*/
    }

    #[test]
    fn test_move_item_middle() {
        assert!(false)
        /*
    // GIVEN a valid view
    let mut container = build_test_container();
    let mut view : ContainerFrameHandler = build_default_container_view(container);
    view.item_list_selection.page_line_count = 4;
    assert_eq!(0, view.item_list_selection.get_true_index());
    let mut contents = view.container.get_contents();
    assert_eq!(4, contents.len());

    // with a series of items
    assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
    assert_eq!("Test Item 2", contents[1].get_self_item().get_name()); // target
    assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[3].get_self_item().get_name());

    // AND we've selected the first item
    view.toggle_select();
    view.toggle_select();

    // WHEN we move down one place and try to move the item (index 1)
    view.move_down();
    view.move_selection();

    // THEN we expect the focused index to remain at the top of the view
    assert_eq!(0, view.item_list_selection.get_true_index());

    // AND the chosen item will be moved to index 1
    let contents = view.container.get_contents();
    assert_eq!(4, contents.len());
    assert_eq!("Test Item 2", contents[0].get_self_item().get_name());
    assert_eq!("Test Item 1", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[3].get_self_item().get_name());*/
    }

    #[test]
    fn test_move_split_items() {
        assert!(false)
        /*
    // GIVEN a valid view
    let mut container = build_test_container();
    let mut view : ContainerFrameHandler = build_default_container_view(container);
    view.item_list_selection.page_line_count = 4;
    assert_eq!(0, view.item_list_selection.get_true_index());
    let mut contents = view.container.get_contents();
    assert_eq!(4, contents.len());

    // with a series of items
    assert_eq!("Test Item 1", contents[0].get_self_item().get_name()); // target
    assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[3].get_self_item().get_name()); // target

    // AND we've selected the first and last item
    view.toggle_select();
    view.toggle_select();
    view.page_down();
    view.toggle_select();
    view.toggle_select();

    // WHEN we move down up one place and try to move the item (index 2)
    view.move_up();
    assert_eq!(2, view.item_list_selection.get_true_index());
    view.move_selection();

    // THEN we expect the focused index to remain at the top of the view
    assert_eq!(0, view.item_list_selection.get_true_index());

    // AND the chosen items will be moved to index 1
    let contents = view.container.get_contents();
    assert_eq!(4, contents.len());
    assert_eq!("Test Item 2", contents[0].get_self_item().get_name());
    assert_eq!("Test Item 1", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 3", contents[3].get_self_item().get_name());*/
    }

    #[test]
    fn test_move_3_split_items() {
        assert!(false)
        /*
    // GIVEN a valid view
    let mut container = build_test_container();
    let mut view : ContainerFrameHandler = build_default_container_view(container);
    view.item_list_selection.page_line_count = 4;
    assert_eq!(0, view.item_list_selection.get_true_index());
    let mut contents = view.container.get_contents();
    assert_eq!(4, contents.len());

    // with a series of items
    assert_eq!("Test Item 1", contents[0].get_self_item().get_name()); // target
    assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[3].get_self_item().get_name()); // target

    // AND we've selected the first 2 and last item
    view.toggle_select();
    view.move_down();
    view.toggle_select();
    view.page_down();
    view.toggle_select();
    view.toggle_select();

    // WHEN we move down up one place and try to move the item (index 2)
    view.move_up();
    assert_eq!(2, view.item_list_selection.get_true_index());
    view.move_selection();

    // THEN we expect the focused index to remain at the top of the view
    assert_eq!(0, view.item_list_selection.get_true_index());

    // AND the chosen items will be moved to index 1
    let contents = view.container.get_contents();
    assert_eq!(4, contents.len());
    assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
    assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
    assert_eq!("Test Item 4", contents[2].get_self_item().get_name());
    assert_eq!("Test Item 3", contents[3].get_self_item().get_name());*/
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
        let data = MoveItemsData { source, to_move, target: Some(target), position: None };
        let data_expected = data.clone();
        let result = move_items(data, &mut level);
        // THEN we expect None to return
        assert!(result.is_none());
    }
}