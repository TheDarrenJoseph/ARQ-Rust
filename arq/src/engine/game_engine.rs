use termion::input::TermRead;
use tui::backend::{Backend};
use tui::layout::{Rect};
use termion::event::Key;
use std::io;
use std::convert::TryInto;
use uuid::Uuid;

use crate::ui;
use crate::ui::{Draw, build_ui};
use crate::settings;
use crate::settings::Toggleable;
use crate::menu;
use crate::menu::{Selection};
use crate::ui::{SettingsMenuChoice, StartMenuChoice};
use crate::view::{View};
use crate::view::map::MapView;
use crate::view::framehandler::character::{CharacterFrameHandler, ViewMode};
use crate::map::map_generator::build_generator;
use crate::terminal::terminal_manager::TerminalManager;
use crate::map::position::{Position, build_rectangular_area};
use crate::character::{Character, build_player};
use crate::widget::character_stat_line::{build_character_stat_line};
use crate::map::objects::container;
use crate::map::objects::container::{ContainerType};
use crate::map::objects::items;
use crate::map::position::Side;
use crate::view::character_info::{CharacterInfoView, CharacterInfoViewFrameHandler, TabChoice};
use crate::engine::level::Level;
use crate::engine::command::input_mapping;
use crate::engine::command::open_command::OpenCommand;
use crate::engine::command::command::Command;
use crate::engine::command::look_command::LookCommand;
use crate::engine::command::inventory_command::InventoryCommand;


pub struct GameEngine<B: 'static + tui::backend::Backend>  {
    terminal_manager : TerminalManager<B>,
    level : Level,
    ui : ui::UI,
    settings : settings::EnumSettings,
    game_running : bool,
}



impl <B : Backend> GameEngine<B> {

    fn draw_settings_menu(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) })
    }

    fn draw_start_menu(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })
    }

    fn handle_settings_menu_selection(&mut self) -> Result<(), io::Error> {
        loop {
            let last_selection = self.ui.settings_menu.selection;
            let key = io::stdin().keys().next().unwrap().unwrap();
            self.ui.settings_menu.handle_input(key);
            let selection = self.ui.settings_menu.selection;
            log::info!("Selection is: {}", selection);
            if last_selection != selection {
                log::info!("Selection changed to: {}", selection);
                self.draw_settings_menu()?;
            }

            if self.ui.settings_menu.exit {
                log::info!("Menu exited.");
                break;
            }

            if self.ui.settings_menu.selected {
                match self.ui.settings_menu.selection.try_into() {
                    Ok(SettingsMenuChoice::FogOfWar) => {
                        match self.settings.settings.iter_mut().find(|x| x.name == "Fog of war") {
                            Some(s) => {
                                s.toggle();
                                log::info!("Fog of war: {}", s.value);
                            },
                            None => {}
                        }
                    },
                    Ok(SettingsMenuChoice::Quit) => {
                        log::info!("Closing settings..");
                        break;
                    },
                    Err(_) => {}
                }
            }
        }
        Ok(())
    }

    fn handle_start_menu_selection(&mut self) -> Result<StartMenuChoice, io::Error> {
        loop {
            let last_selection = self.ui.start_menu.selection;
            let key = io::stdin().keys().next().unwrap().unwrap();
            self.ui.start_menu.handle_input(key);
            let selection = self.ui.start_menu.selection;
            log::info!("Selection is: {}", selection);
            if last_selection != selection {
                log::info!("Selection changed to: {}", selection);
                let ui = &mut self.ui;
                self.terminal_manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })?;
            }

            if self.ui.start_menu.exit {
                log::info!("Menu exited.");
                return Ok(StartMenuChoice::Quit);
            }

            if self.ui.start_menu.selected {
                match self.ui.start_menu.selection.try_into() {
                    Ok(x) => {
                        return Ok(x);
                    },
                    Err(_) => {}
                }
            }
        }
    }

    pub fn start_menu(&mut self) -> Result<(), io::Error> {
        loop {
            // Hide additional widgets when paused
            self.ui.render_additional = false;
            self.draw_start_menu()?;
            let start_choice = self.handle_start_menu_selection()?;
            match start_choice {
                StartMenuChoice::Play => {
                    self.ui.render_additional = true;
                    if !self.game_running {
                        log::info!("Starting game..");
                        self.start_game()?;
                        break;
                    } else {
                        return Ok(());
                    }
                },
                StartMenuChoice::Settings => {
                    log::info!("Showing settings..");
                    let ui = &mut self.ui;
                    self.terminal_manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) })?;
                    self.handle_settings_menu_selection()?;
                },
                StartMenuChoice::Info => {
                    log::info!("Showing info..");
                    let ui = &mut self.ui;
                    self.terminal_manager.terminal.draw(|frame| { ui.draw_info(frame) })?;
                    io::stdin().keys().next();
                },
                StartMenuChoice::Quit => {
                    if self.game_running {
                        self.game_running = false;
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    fn build_characters(&self) -> Vec<Character> {
        let position = Position { x: 1, y: 1};
        let player = build_player("Player".to_string(), position);

        let mut characters = Vec::new();
        characters.push(player);
        return characters;
    }

    fn initialise_characters(&mut self) {
        let mut characters = self.build_characters();
        let mut character_view = CharacterFrameHandler { character: characters.get(0).unwrap().clone(),  widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::CREATION};
        // Being capture of a new character
        /**
        let mut character_creation_result = InputResult { generic_input_result:
            GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        };**/
        //while !character_creation_result.generic_input_result.done {
            let ui = &mut self.ui;
            let mut frame_area = Rect::default();

            self.terminal_manager.terminal.draw(|frame| {
                //ui.render(frame);
                //character_view.handle_frame(frame, FrameData { data: characters.get(0).unwrap().clone(), frame_size: frame_area });
            });

            //let key = io::stdin().keys().next().unwrap().unwrap();
            //character_creation_result = character_view.handle_input(Some(key)).unwrap();
        //}
        let mut updated_character = character_view.get_character();

        // Grab the first room and set the player's position there
        if let Some(map) = &self.level.map {
            let room = &map.get_rooms()[0];
            let area = room.get_inside_area();
            let start_position = area.start_position;
            updated_character.set_position(start_position);
        }

        characters[0] = updated_character;
        self.level.characters = characters.clone();
        self.build_testing_inventory();
    }

    // TODO refactor into a singular component shared with commands
    fn re_render(&mut self) -> Result<(), io::Error>  {
        let mut ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn draw_map_view(&mut self) -> Result<(), io::Error> {
        match &mut self.level.map {
            Some(m) => {
                if let Some(frame_size) = self.ui.frame_size {
                    let mut map_view = MapView { map: m, characters: self.level.characters.clone(), ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, view_area: None };

                    // Adjust the map view size to fit within our borders / make space for the console
                    let map_view_start_pos = Position { x : frame_size.start_position.x + 1, y: frame_size.start_position.y + 1};
                    let map_view_frame_size = Some(build_rectangular_area(map_view_start_pos, frame_size.size_x - 2, frame_size.size_y - 8 ));
                    map_view.draw(map_view_frame_size)?;
                    map_view.draw_containers()?;
                    map_view.draw_characters()?;
                    self.ui.console_print("Arrow keys to move.".to_string());
                    self.re_render();
                }
            },
            None => {}
        }
        Ok(())
    }

    fn start_game(&mut self) -> Result<(), io::Error>{
        self.ui.start_menu = menu::build_start_menu(true);
        let map_area = build_rectangular_area(Position { x: 0, y: 0 }, 40, 20);
        let mut map_generator = build_generator(map_area);
        self.level.map = Some(map_generator.generate());

        self.initialise_characters();

        self.game_running = true;
        while self.game_running {
            if self.ui.additional_widgets.is_empty() {
                let player = self.level.get_player_mut();
                let stat_line = build_character_stat_line(player.get_health(), player.get_details(), player.get_inventory().get_loot_value());
                self.ui.additional_widgets.push(stat_line);
            }

            self.ui.show_console();
            self.draw_map_view();
            self.game_loop()?;
        }
        //self.terminal_manager.terminal.clear()?;
        Ok(())
    }

    fn build_testing_inventory(&mut self) {
        let inventory = self.level.characters[0].get_inventory();
        let gold_bar = items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        inventory.add_item(gold_bar);

        let silver_bar = items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        inventory.add_item(silver_bar);

        let bronze_bar = items::build_item(Uuid::new_v4(), "Bronze Bar".to_owned(), 'X', 1, 50);
        let mut bag = container::build(Uuid::new_v4(), "Bag".to_owned(), '$', 5, 50, ContainerType::OBJECT, 50);
        let carton = container::build(Uuid::new_v4(), "Carton".to_owned(), '$', 1, 50, ContainerType::OBJECT, 5);
        bag.add(carton);
        bag.add_item(bronze_bar);

        for i in 1..=30 {
            let test_item = items::build_item(Uuid::new_v4(), format!("Test Item {}", i), 'X', 1, 100);
            inventory.add_item(test_item);
        }

        inventory.add(bag);
    }

    fn handle_player_movement(&mut self, side: Side) {
        let mut updated_position = self.level.find_player_side_position(side);
        let map = self.level.get_map();
        match updated_position {
            Some(pos) => {
                if let Some(m) = map {
                    if m.is_traversable(pos) {
                        let player = self.level.get_player_mut();
                        player.set_position(pos);
                    }
                }
            },
            None => {}
        }
    }

    pub fn menu_command(&mut self) -> Result<(), io::Error> {
        self.terminal_manager.terminal.clear()?;
        self.ui.hide_console();
        self.start_menu()?;
        self.ui.show_console();
        self.terminal_manager.terminal.clear()?;
        Ok(())
    }

    pub fn handle_input(&mut self, key : Key) -> Result<(), io::Error>  {
        match key {
            Key::Char('q') => {
                self.menu_command();
            },
            Key::Char('i') => {
                let mut command = InventoryCommand { level: &mut self.level, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager };
                command.handle(key);
            },
            Key::Char('k') => {
                let mut command = LookCommand { level: &mut self.level, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager };
                command.handle(key);
            },
            Key::Char('o') => {
                self.ui.console_print("What do you want to open?. Arrow keys to choose. Repeat command to choose current location.".to_string());
                self.re_render();
                let mut command = OpenCommand { level: &mut self.level, ui: &mut self.ui, terminal_manager: &mut self.terminal_manager };
                command.handle(key);
            },
            Key::Down | Key::Up | Key::Left | Key::Right => {
                if let Some(side) = input_mapping::key_to_side(key) {
                    self.handle_player_movement(side);
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn game_loop(&mut self) -> Result<(), io::Error> {
        let key = io::stdin().keys().next().unwrap().unwrap();
        //self.terminal_manager.terminal.clear()?;
        self.handle_input(key);
        //self.terminal_manager.terminal.clear()?;
        Ok(())
    }
}

pub fn build_game_engine<'a, B: tui::backend::Backend>(mut terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, io::Error> {
    let ui = build_ui();
    let fog_of_war = settings::Setting { name : "Fog of war".to_string(), value : false };
    let settings = settings::EnumSettings { settings: vec![fog_of_war] };
    Ok(GameEngine { terminal_manager, level: Level { map: None, characters: Vec::new()}, ui, settings, game_running: false})
}


#[cfg(test)]
mod tests {
    use termion::input::TermRead;
    use termion::raw::RawTerminal;
    use tui::backend::TermionBackend;
    use termion::event::Key;
    use std::io;
    use std::convert::TryInto;
    use uuid::Uuid;
    use std::collections::HashMap;

    use crate::terminal::terminal_manager;
    use crate::ui;
    use crate::ui::Draw;
    use crate::settings;
    use crate::settings::Toggleable;
    use crate::menu;
    use crate::menu::{Selection};
    use crate::ui::{SettingsMenuChoice, StartMenuChoice};
    use crate::view::View;
    use crate::view::map::MapView;
    use crate::view::framehandler::character::{CharacterFrameHandler, ViewMode};
    use crate::view::framehandler::container::{ContainerFrameHandler, build_container_view};
    use crate::map::map_generator::build_generator;
    use crate::map::Map;
    use crate::terminal::terminal_manager::TerminalManager;
    use crate::map::position::{Position, build_rectangular_area};
    use crate::character::{Character, build_player};
    use crate::widget::character_stat_line::{build_character_stat_line, CharacterStatLineState};
    use crate::map::objects::container;
    use crate::map::objects::container::ContainerType;
    use crate::list_selection::build_list_selection;
    use crate::map::objects::items;
    use crate::map::position::Side;
    use crate::map::position::{Area, build_square_area};
    use crate::map::tile::{Tile, TileDetails, build_library};
    use crate::engine::game_engine::*;

    fn build_tiles(map_area: Area, tile : Tile) -> Vec<Vec<TileDetails>> {
        let tile_library = build_library();
        let mut map_tiles = Vec::new();
        let mut row;
        for y in map_area.start_position.y..=map_area.end_position.y {
            row = Vec::new();
            for x in map_area.start_position.x..=map_area.end_position.x {
                row.push( tile_library[&tile].clone());
            }
            map_tiles.push(row);
        }
        map_tiles
    }

    fn build_characters(player_position : Position) -> Vec<Character> {
        let player = build_player("Player".to_string(), player_position);
        vec![player]
    }

    fn test_movement_input(map: Map, start_position: Position, input: Vec<Key>, expected_end_position : Position) {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let terminal_manager = terminal_manager::init_test().unwrap();
        let mut game_engine = build_game_engine(terminal_manager);

        match game_engine {
            Result::Ok(mut engine) => {
                let characters = build_characters(start_position);
                engine.level.set_characters(characters);
                engine.level.set_map(Some(map));

                // AND The player is placed in the middle of the map
                assert_eq!(start_position, engine.level.get_player().get_position());

                // WHEN we push the down key
                for key in input {
                    engine.handle_input(key);
                }

                // THEN we expect the player to be moved into the traversable tile
                assert_eq!(expected_end_position, engine.level.get_player().get_position());
            },
            _ => {
                panic!("Expected a valid Game Engine instance!")
            }
        }
    }

    #[test]
    fn test_build_game_engine() {
        let terminal_manager = terminal_manager::init_test().unwrap();
        let game_engine = build_game_engine(terminal_manager);
    }

    #[test]
    fn test_move_down_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND the middle / bottom middle tile is a corridor
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        map_tiles[2] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move down
        let input = vec![Key::Down];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:1, y: 2};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_down_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND only the middle tile is a corridor
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move down
        let input = vec![Key::Down];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_up_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND the middle / bottom middle tile is a corridor
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        map_tiles[2] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the bottom middle of the map
        let start_position = Position{x:1, y:2};
        // WHEN we attempt to move up
        let input = vec![Key::Up];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_up_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND only the middle end tile is a corridor
        map_tiles[2] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle end of the map
        let start_position = Position{x:1, y:2};
        // WHEN we attempt to move up
        let input = vec![Key::Up];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 2};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_left_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND the middle / middle left tile is a corridor
        map_tiles[1] [0] = tile_library[&Tile::Corridor].clone();
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move left
        let input = vec![Key::Left];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:0, y: 1};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_left_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND only the middle end tile is a corridor
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move left
        let input = vec![Key::Left];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_right_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND the middle / middle right tile is a corridor
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        map_tiles[1] [2] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:2, y: 1};
        test_movement_input(map, start_position, input, expected_end_position);
    }

    #[test]
    fn test_move_right_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), Tile::NoTile);

        // AND only the middle end tile is a corridor
        map_tiles[1] [1] = tile_library[&Tile::Corridor].clone();
        let map = crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(map, start_position, input, expected_end_position);
    }

}