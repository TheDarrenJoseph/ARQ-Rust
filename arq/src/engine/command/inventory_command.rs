use std::io;
use termion::event::Key;
use std::collections::HashMap;
use termion::input::TermRead;
use std::collections::HashSet;

use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::view::View;
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, ContainerFrameHandlerCommand};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui;
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{DropItems, MoveItems, TakeItems};
use crate::view::framehandler::container::ContainerFrameHandlerCommand::{OPEN, TAKE, DROP};
use crate::view::character_info::{CharacterInfoViewFrameHandler, CharacterInfoView, TabChoice};
use crate::engine::command::command::Command;
use crate::engine::container_util;
use crate::ui::Draw;
use crate::view::callback::Callback;

pub struct InventoryCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

struct CallbackState<'a> {
    pub level : &'a mut Level,
    pub container: &'a mut Container,
    pub data : ContainerFrameHandlerInputResult
}

fn drop_items(items: Vec<Item>, state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    let position = state.level.get_player_mut().get_position().clone();
    log::info!("InventoryCommand - Dropping {} items at position: {}, {}", items.len(),  position.x, position.y);

    // Find the container on the map and add the "container" wrappers there
    let mut undropped = Vec::new();
    if let Some(m) = state.level.get_map_mut() {
        if let Some(mut pos_container) = m.find_container_mut(position) {
            for item in items {
                // Find the "container" wrappper matching the item returned
                if let Some(container_item) = state.container.find_mut(&item) {
                    let dropping_container_item = container_item.clone();
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

fn handle_callback(state: CallbackState) -> Option<ContainerFrameHandlerInputResult> {
    match state.data {
        DropItems(ref items) => {
            log::info!("[inventory command] Received data for DropItems with {} items", items.len());
            return drop_items(items.to_vec(), state);
        },
        MoveItems(mut data) => {
            log::info!("[inventory command] Received data for MoveItems with {} items", data.to_move.len());
            return container_util::move_player_items(data, state.level);
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
        let mut commands: HashSet<ContainerFrameHandlerCommand> = HashSet::new();

        let ui = &mut self.ui;
        let terminal_manager = &mut self.terminal_manager;
        let frame_handler = CharacterInfoViewFrameHandler { tab_choice: TabChoice::INVENTORY, container_views: Vec::new(), character_view: None };

        self.ui.console_print("Up/Down - Move\nEnter - Toggle selection".to_string());

        let level = &mut self.level;
        let player = &mut level.characters[0].clone();
        let updated_inventory;
        {
            let mut character_info_view = CharacterInfoView { character: player, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, frame_handler, callback: Box::new(|data| {None}) };
            character_info_view.set_callback(Box::new(|data| {
                handle_callback(CallbackState { level, container: &mut callback_container, data })
            }));
            character_info_view.begin();
            updated_inventory = character_info_view.frame_handler.container_views.get(0).unwrap().container.clone();
        }
        level.characters[0].set_inventory(updated_inventory);
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
    use crate::view::framehandler::container::{ContainerFrameHandler, build_container_view, build_default_container_view, Column, ContainerFrameHandlerInputResult};
    use crate::terminal::terminal_manager::TerminalManager;
    use crate::ui::{UI, build_ui};
    use crate::list_selection::ListSelection;
    use crate::view::framehandler::console::{ConsoleFrameHandler, ConsoleBuffer};
    use crate::map::tile::{Colour, Tile};
    use crate::engine::command::inventory_command::{InventoryCommand, handle_callback, CallbackState};
    use crate::engine::level::Level;
    use crate::character::build_player;
    use crate::map::position::{Position, build_square_area};

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
        let mut view_result = ContainerFrameHandlerInputResult::DropItems(selected_container_items);
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
        let mut view_result = ContainerFrameHandlerInputResult::DropItems(selected_container_items);
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
        let mut view_result = ContainerFrameHandlerInputResult::DropItems(selected_container_items);
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