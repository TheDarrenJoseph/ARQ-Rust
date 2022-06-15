use std::io;
use std::io::{Error, ErrorKind};
use termion::event::Key;
use termion::input::TermRead;

use crate::view::framehandler::container;
use crate::engine::command::command::Command;
use crate::view::world_container::{WorldContainerViewFrameHandlers, WorldContainerView};
use crate::view::framehandler::container::ContainerFrameHandlerInputResult;
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::TakeItems;
use crate::map::position::Position;
use crate::view::callback::Callback;
use crate::view::View;
use crate::map::Map;
use crate::engine::level::Level;
use crate::ui;
use crate::map::objects::container::Container;
use crate::engine::command::input_mapping;
use crate::map::objects::container::ContainerType::{AREA, ITEM, OBJECT};
use crate::map::room::Room;
use crate::terminal::terminal_manager::TerminalManager;

pub struct LookCommand<'a, B: 'static + tui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut ui::UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
}

fn describe_position_in_room(pos: Position, room: &Room) -> Option<String> {
    if let Some(door) = &room.doors.iter().find(|d| d.position == pos) {
        log::info!("Position is a door.");
        return Some(format!("There's a {} here.", &door.tile_details.name));
    }
    None
}

fn describe_position_container(c: &Container) -> Result<String, io::Error> {
    let item_count = c.get_item_count();
    let container_type = c.get_container_type();

    if container_type != AREA  {
        return Err(Error::new(ErrorKind::Other, format!("Unexpected input! Cannot describe position with container of type {}.", container_type)))
    }

    let c_item_name = c.get_self_item().get_name();
    if item_count > 1 {
        Ok(format!("There's {} items on the {} here.", item_count, c_item_name))
    } else if item_count == 1 {
        let top_item_name = c.get_contents()[0].get_self_item().get_name();
        Ok(format!("There's a {} on the {} here.", top_item_name, c_item_name))
    } else {
        let item_name = c.get_self_item().get_name();
        Ok(format!("The {} is empty here.", item_name))
    }
}

fn describe_position(pos: Position, level : &mut Level) -> Result<String, io::Error> {
    if let Some(map) = &level.map {
        if let Some(room) = map.get_rooms().iter().find(|r| r.area.contains_position(pos)) {
            log::info!("Position is in a room.");
            let prompt = describe_position_in_room(pos, room);
            return if prompt.is_some() {
                Ok(prompt.unwrap())
            } else {
                Ok("There's nothing here.".to_string())
            }
        }

        return if let Some(c) = map.containers.get(&pos) {
            describe_position_container(c)
        } else {
            Ok("There's nothing here.".to_string())
        }
    } else {
        log::error!("Look command failure, no map on level!");
        return Err(Error::new(ErrorKind::Other, "Look command failure, no map on level!"))
    }
}

impl <B: tui::backend::Backend> LookCommand<'_, B> {
    // TODO refactor alongside other commands / engine func
    fn re_render(&mut self) -> Result<(), io::Error> {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn print(&mut self, prompt: String) -> Result<(), io::Error> {
        self.ui.console_print(prompt);
        self.re_render();
        Ok(())
    }
}

impl <B: tui::backend::Backend> Command for LookCommand<'_, B> {
    fn handles_key(&self, key: Key) -> bool {
        return match key {
            Key::Char('k') => {
                true
            }
            _ => {
                false
            }
        };
    }

    fn handle(&mut self, command_key: Key) -> Result<(), io::Error> {
        self.ui.console_print("Where do you want to look?. Arrow keys to choose. Repeat command to choose current location.".to_string());
        self.re_render();
        let key = io::stdin().keys().next().unwrap().unwrap();
        let position = self.level.find_adjacent_player_position(key, command_key);
        if let Some(p) = position {
            log::info!("Player looking at map position: {}, {}", &p.x, &p.y);
            self.re_render();
            let prompt = describe_position(p, &mut self.level);
            if prompt.is_ok() {
                self.print(prompt.unwrap());
                let key = io::stdin().keys().next().unwrap().unwrap();
            } else {
                return Err(prompt.unwrap_err())
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use uuid::Uuid;
    use crate::engine::command::look_command::describe_position;
    use crate::engine::level::Level;
    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::Tile;
    use crate::view::framehandler::character;
    use crate::character::build_player;

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
    fn test_describe_area_multiple_items() {
        // GIVEN a valid map
        // that holds a source container (AREA) containing 3 OBJECT containers
        let mut source_container =  build(Uuid::new_v4(), "Floor".to_owned(), 'X', 1, 1, ContainerType::AREA, 100);
        let container1 =  build(Uuid::new_v4(), "Test Container 1".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container2 =  build(Uuid::new_v4(), "Test Container 2".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container3 =  build(Uuid::new_v4(), "Test Container 3".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        source_container.push(vec![container1, container2, container3]);
        assert_eq!(3, source_container.get_item_count());

        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to describe the container position
        let prompt = describe_position(container_pos, &mut level);

        // THEN we expect
        assert!(prompt.is_ok());
        assert_eq!("There's 3 items on the Floor here.", prompt.unwrap());
    }

    #[test]
    fn test_describe_area_single_item() {
        // GIVEN a valid map
        // that holds a source container (AREA) containing 1 OBJECT container
        let mut source_container =  build(Uuid::new_v4(), "Floor".to_owned(), 'X', 1, 1, ContainerType::AREA, 100);
        let bag =  build(Uuid::new_v4(), "Bag".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        source_container.push(vec![bag]);
        assert_eq!(1, source_container.get_item_count());
        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to describe the container position
        let prompt = describe_position(container_pos, &mut level);

        // THEN we expect the single item and area to be described
        assert!(prompt.is_ok());
        assert_eq!("There's a Bag on the Floor here.", prompt.unwrap());
    }

    #[test]
    fn test_describe_area_empty() {
        // GIVEN a valid map
        // that holds a source container (AREA) containing nothing
        let mut source_container =  build(Uuid::new_v4(), "Floor".to_owned(), 'X', 1, 1, ContainerType::AREA, 100);
        assert_eq!(0, source_container.get_item_count());
        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to describe the container position
        let prompt = describe_position(container_pos, &mut level);

        // THEN we expect the area to be empty
        assert!(prompt.is_ok());
        assert_eq!("The Floor is empty here.", prompt.unwrap());
    }

    #[test]
    fn test_describe_area_nothing() {
        // GIVEN a valid map
        // that holds a source container (AREA) containing 1 OBJECT container
        let mut source_container =  build(Uuid::new_v4(), "Floor".to_owned(), 'X', 1, 1, ContainerType::AREA, 100);
        let bag =  build(Uuid::new_v4(), "Bag".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        source_container.push(vec![bag]);
        assert_eq!(1, source_container.get_item_count());
        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to describe a different position
        let prompt = describe_position( Position { x: 1, y: 2}, &mut level);

        // THEN we expect nothing to be there
        assert!(prompt.is_ok());
        assert_eq!("There's nothing here.", prompt.unwrap());
    }

    #[test]
    fn test_describe_area_invalid_container() {
        // GIVEN a valid map
        // that holds a source container (AREA) containing 1 OBJECT container
        let mut source_container =  build(Uuid::new_v4(), "Floor".to_owned(), 'X', 1, 1, ContainerType::AREA, 100);
        let bag =  build(Uuid::new_v4(), "Bag".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        source_container.push(vec![bag]);
        assert_eq!(1, source_container.get_item_count());
        let source = source_container.clone();
        let container_pos =  Position { x: 1, y: 1};
        let mut level = build_test_level(container_pos, source_container);

        // WHEN we call to describe a different position
        let prompt = describe_position( Position { x: 0, y: 0}, &mut level);

        // THEN we expect nothing to be there
        assert!(prompt.is_ok());
        assert_eq!("There's nothing here.", prompt.unwrap());
    }
}