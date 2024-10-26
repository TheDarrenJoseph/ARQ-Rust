use std::collections::HashMap;
use std::io;
use std::io::Error;

use termion::event::Key;
use termion::input::TermRead;

use crate::engine::command::command::Command;
use crate::engine::container_util;
use crate::engine::engine_helpers::input_handler;
use crate::engine::level::Level;
use crate::error::errors::{error_result, ErrorWrapper};
use crate::input::{IoKeyInputResolver, KeyInputResolver, MockKeyInputResolver};
use crate::map::objects::container::Container;
use crate::map::position::Position;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::bindings::action_bindings::Action;
use crate::ui::bindings::open_bindings::{map_open_input_to_side, OpenInput};
use crate::ui::ui::UI;
use crate::view::framehandler::container;
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::{MoveItems, MoveToContainerChoice, TakeItems};
use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, MoveItemsData, MoveToContainerChoiceData};
use crate::view::model::usage_line::{UsageCommand, UsageLine};
use crate::view::util::callback::Callback;
use crate::view::world_container_view::{WorldContainerView, WorldContainerViewFrameHandlers};
use crate::view::{InputHandler, InputResult, View};

pub struct OpenCommand<'a, B: 'static + ratatui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub input_resolver: Box<dyn KeyInputResolver>
}

const UI_USAGE_HINT: &str = "Up/Down - Move\nEnter/q - Toggle/clear selection\nEsc - Exit";
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

fn build_container_choices<'a>(data: &'a MoveToContainerChoiceData, level: &'a mut Level) -> Result<ContainerFrameHandlerInputResult, ErrorWrapper> {
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
                return ErrorWrapper::internal_result(format!("Failed build container choices: {}", sub_containers_result.err().unwrap()));
            }
        } else {
            return ErrorWrapper::internal_result( format!("Cannot build container choices. Cannot find container at position: {:?}", pos));
        }
    } else {
        return ErrorWrapper::internal_result(String::from("Cannot build container choices. No position provided."));
    }
}

impl <B: ratatui::backend::Backend> OpenCommand<'_, B> {

    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn open_container(&mut self, p: Position, c: &Container) -> Result<InputResult<bool>, ErrorWrapper> {
        self.ui.set_console_buffer(UI_USAGE_HINT.to_string());

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
        
        let mock_input_resolver = &mut self.input_resolver.as_any_mut().downcast_mut::<MockKeyInputResolver>();
        
        let mut input_handler : Box<dyn KeyInputResolver> = Box::new(IoKeyInputResolver{});
        if let Some(mock) = mock_input_resolver {
            input_handler = Box::new(MockKeyInputResolver { key_results: mock.key_results.clone() });
        }
        
        let mut world_container_view = WorldContainerView {
            ui,
            terminal_manager,
            frame_handlers: frame_handler,
            container: view_container,
            callback: Box::new(|_data| {None}),
            input_resolver: input_handler
        };
        world_container_view.set_callback(Box::new(|input_result| {
            return handle_callback(level, p.clone(), input_result);
        }));
        world_container_view.begin()
    }
}

impl <B: ratatui::backend::Backend> Command<OpenInput> for OpenCommand<'_, B> {
    fn can_handle_action(&self, action: Action) -> bool {
        return match action {
            Action::OpenNearby => {
                true
            }
            _ => {
                false
            }
        };
    }

    fn handle_input(&mut self, input: Option<&OpenInput>) -> Result<(), ErrorWrapper> {
        let mut message = NOTHING_ERROR.to_string();
        let side = map_open_input_to_side(input);
        if let Some(p) = self.level.find_adjacent_player_position(side) {
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
                            
                            // Automatically open any fixed container if it's the only item in this area container
                            // For example, a single Chest in the Floor container
                            let contains_single_container = item_count == 1 && c.get_contents()[0].is_fixed_container();
                            if contains_single_container && c.get_contents()[0].get_top_level_count() > 0 {
                                to_open = Some(c.get_contents()[0].clone());
                            } else {
                                // Otherwise, show everything in this area container
                                to_open = Some(c.clone());
                            }
                        }
                    }
                }
            }

            if let Some(c) = to_open {
                self.ui.clear_console_buffer();
                self.re_render()?;
                log::info!("Player opening container of type {:?} and length: {}", c.container_type, c.get_total_count());
                self.open_container(p.clone(), &c)?;
            } else {
                return ErrorWrapper::internal_result(message)
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use termion::event::Key;
    use ratatui::backend::TestBackend;

    use uuid::Uuid;

    use crate::character::character_details::build_default_character_details;
    use crate::character::Character;
    use crate::engine::command::command::Command;
    use crate::engine::command::open_command::{handle_callback, OpenCommand};
    use crate::engine::game_engine::build_test_game_engine;
    use crate::input::MockKeyInputResolver;
    use crate::map::objects::container::{Container, ContainerType};
    use crate::map::objects::items::Item;
    use crate::map::position::{Area, Position};
    use crate::map::tile::{Colour, Symbol};
    use crate::terminal::terminal_manager;
    use crate::test::utils::test_utils::{build_test_level, build_test_levels_for_level};
    use crate::ui::bindings::open_bindings::OpenInput::OpenRight;
    use crate::view::framehandler::container::{ContainerFrameHandlerInputResult, TakeItemsData};
    use crate::view::MIN_RESOLUTION;

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
            container.add_item(test_item).expect(format!("Test Item {} should have been added to the test container!", i).as_str());
        }

        assert_eq!(ContainerType::OBJECT, container.container_type);
        assert_eq!(100, container.get_weight_limit());
        let contents = container.get_contents();
        assert_eq!(4, contents.len());
        container
    }
    
    fn choose_n_items(container: &Container, count: i32) -> Vec<Item> {
        let mut selected_container_items = Vec::new();
        for i in 0..count {
            selected_container_items.push(container.get(i).get_self_item().clone());
        }
        assert_eq!(count as usize, selected_container_items.len());
        selected_container_items
    }
    
    #[test]
    fn test_take_callback() {
        // GIVEN a valid level with an player inventory to extract items into
        let container_pos =  Position { x: 0, y: 0};
        
        // AND we've selected the first 2 items to take 
        let container = build_test_container();
        let mut level = build_test_level(Some((container_pos, container.clone())), None);

        let initial_top_level_item_count = container.get_contents().len();
        let initial_top_level_inventory_item_count = level.characters.get_player_mut().unwrap().get_inventory_mut().get_contents().len();
        assert_eq!(4, initial_top_level_item_count);
        assert_eq!(62, initial_top_level_inventory_item_count);
        
        let mut selected_container_items = choose_n_items(&container, 2);
        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        
        let data = TakeItemsData { source: container.clone(), to_take: selected_container_items, position: Some(container_pos) };
        let view_result = ContainerFrameHandlerInputResult::TakeItems(data);

        // WHEN we call to handle a take callback with 2 of the items
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
        let updated_inventory = inventory.get_contents();
        assert_eq!(initial_top_level_inventory_item_count + 2, updated_inventory.len());
        // AND the bottom 2 items should be the moved items
        assert_eq!(chosen_item_1, *updated_inventory.get(initial_top_level_inventory_item_count).unwrap().get_self_item());
        assert_eq!(chosen_item_2, *updated_inventory.get(initial_top_level_inventory_item_count + 1).unwrap().get_self_item());
        
        // And the container we took from should have 2 less items in the top level
        let updated_container = level.get_map_mut().unwrap().find_container(&container, container_pos).unwrap();
        assert_eq!(2, updated_container.get_contents().len());
    }

    #[test]
    fn test_take_callback_too_many_items() {
        // GIVEN a valid map with a player inventory to extract items into
        // AND the inventory only has space for 2 items
        let inventory = Container::new(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1.0, 1, ContainerType::OBJECT, 2);
        let _character_details = build_default_character_details();
        let player = Character::new(String::from("Test Player"), Position { x: 0, y: 0}, Symbol::new('@', Colour::Green), inventory);
        let container_pos =  Position { x: 0, y: 0};
        
        // AND we've selected 3 items to take (with only space for 2 of them)
        let container = build_test_container();
        let _callback_container = container.clone();
        let mut selected_container_items   = choose_n_items(&container, 3);

        let mut level = build_test_level(Some((container_pos, container.clone())), Some(player));

        let chosen_item_1 = selected_container_items.get(0).unwrap().clone();
        let chosen_item_2 = selected_container_items.get(1).unwrap().clone();
        let chosen_item_3 = selected_container_items.get(2).unwrap().clone();
        let data = TakeItemsData { source: container, to_take: selected_container_items, position: Some(container_pos) };
        let view_result = ContainerFrameHandlerInputResult::TakeItems(data);
        
        // WHEN we call to drop these items
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

    #[test]
    fn test_handle() {
        // GIVEN a test game engine, level, player, and container
        let container = build_test_container();
        let container_pos = Position::new(1, 0);
        let level = build_test_level(Some((container_pos, container)), None);

        let levels = build_test_levels_for_level(level);

        let mut terminal_manager = terminal_manager::init_test(MIN_RESOLUTION).unwrap();
        let mut game_engine = build_test_game_engine(levels, terminal_manager).unwrap();

        // AND we've initialised the UI areas
        game_engine.ui_wrapper.ui.init(Area::from_resolution(MIN_RESOLUTION));
        
        // AND we have an OpenCommand with all this data
        // And our mocked input will return Escape to quit the view immediately
        let key_results = vec![Key::Esc];
        let mut command = OpenCommand { level: game_engine.levels.get_level_mut(), ui: &mut game_engine.ui_wrapper.ui, terminal_manager: &mut game_engine.ui_wrapper.terminal_manager, input_resolver: Box::new(MockKeyInputResolver { key_results }) };
        
        // WHEN we call to handle the opening of a container
        // Player is at 0,0. Container is at 0,1 to the right of the player
        // TODO mock input from keyboard to escape view\
        let input = Some(&OpenRight);
        command.handle_input(input).expect("Open command should open container");
        
        // THEN we expect to reach this point successfully
    }
}