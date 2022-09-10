use std::convert::TryInto;
use std::io;

use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;
use tui::layout::Rect;

use crate::character::{build_player, Character};
use crate::engine::command::command::Command;
use crate::engine::command::input_mapping;
use crate::engine::command::inventory_command::InventoryCommand;
use crate::engine::command::look_command::LookCommand;
use crate::engine::command::open_command::OpenCommand;
use crate::engine::level::Level;
use crate::map::map_generator::{build_dev_chest, build_dev_inventory, build_generator};
use crate::map::objects::container;
use crate::map::objects::container::ContainerType;
use crate::map::objects::items;
use crate::map::position::{build_rectangular_area, Position};
use crate::map::position::Side;
use crate::menu;
use crate::menu::Selection;
use crate::settings;
use crate::settings::Toggleable;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui;
use crate::ui::{build_ui, Draw, FrameData, FrameHandler};
use crate::ui::{SettingsMenuChoice, StartMenuChoice};
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::framehandler::character::{CharacterFrameHandler, ViewMode};
use crate::view::framehandler::character::CharacterFrameHandlerInputResult::VALIDATION;
use crate::view::map::MapView;
use crate::widget::character_stat_line::build_character_stat_line;

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

    // TODO this should live in it's own view likely
    // Shows character creation screen
    // Returns the finished character once input is confirmed
    fn show_character_creation(&mut self, base_character: Character) -> Result<Character, io::Error> {
        let mut character_view = CharacterFrameHandler { character: base_character.clone(),  widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::CREATION};
        // Begin capture of a new character
        let mut character_creation_result = InputResult { generic_input_result:
        GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        };
        while !character_creation_result.generic_input_result.done {
            let ui = &mut self.ui;
            ui.show_console();
            self.terminal_manager.terminal.draw(|frame| {
                let areas: Vec<Rect> = ui.get_view_areas(frame.size());
                let mut main_area = areas[0];
                main_area.height -= 2;
                ui.render(frame);
                character_view.handle_frame(frame, FrameData { data: base_character.clone(), frame_size: main_area });
            });
            ui.hide_console();

            let key = io::stdin().keys().next().unwrap().unwrap();
            character_creation_result = character_view.handle_input(Some(key))?;

            match character_creation_result.view_specific_result {
                Some(VALIDATION(message)) => {
                    ui.console_print(message);
                    self.re_render();
                },
                Some(NONE) => {
                    return Ok(character_view.get_character())
                },
                _ => {}
            }
        }
        return Ok(character_view.get_character());
    }

    fn initialise_characters(&mut self) -> Result<(), io::Error> {
        let mut characters = self.build_characters();
        let mut player = characters.get_mut(0).unwrap();
        // Uncomment to use character creation
        //let mut updated_character = self.show_character_creation(characters.get(0).unwrap().clone())?;
        // Grab the first room and set the player's position there
        if let Some(map) = &self.level.map {
            let room = &map.get_rooms()[0];
            let area = room.get_inside_area();
            let start_position = area.start_position;
            player.set_position(start_position);
        }
        self.level.characters = characters.clone();
        self.build_testing_inventory();
        return Ok(());
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

        self.initialise_characters()?;

        self.game_running = true;
        while self.game_running {
            if self.ui.additional_widgets.is_empty() {
                let player = self.level.get_player_mut();
                let stat_line = build_character_stat_line(player.get_health(), player.get_details(), player.get_inventory_mut().get_loot_value());
                self.ui.additional_widgets.push(stat_line);
            }

            self.ui.show_console();

            match self.draw_map_view() {
                Err(e) => {
                    log::error!("Error when attempting to draw map: {}", e);
                    return Err(e);
                }
                _ => {
                }
            }
            self.game_loop()?;
        }
        //self.terminal_manager.terminal.clear()?;
        Ok(())
    }

    fn build_testing_inventory(&mut self) {
        self.level.characters[0].set_inventory(build_dev_inventory());
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
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::io;

    use termion::event::Key;
    use termion::input::TermRead;
    use termion::raw::RawTerminal;
    use tui::backend::TermionBackend;
    use uuid::Uuid;

    use crate::character::{build_player, Character};
    use crate::engine::game_engine::*;
    use crate::list_selection::build_list_selection;
    use crate::map::Map;
    use crate::map::map_generator::build_generator;
    use crate::map::objects::container;
    use crate::map::objects::container::ContainerType;
    use crate::map::objects::items;
    use crate::map::position::{build_rectangular_area, Position};
    use crate::map::position::{Area, build_square_area};
    use crate::map::position::Side;
    use crate::map::tile::{build_library, Tile, TileDetails};
    use crate::menu;
    use crate::menu::Selection;
    use crate::settings;
    use crate::settings::Toggleable;
    use crate::terminal::terminal_manager;
    use crate::terminal::terminal_manager::TerminalManager;
    use crate::ui;
    use crate::ui::{SettingsMenuChoice, StartMenuChoice};
    use crate::ui::Draw;
    use crate::view::framehandler::character::{CharacterFrameHandler, ViewMode};
    use crate::view::framehandler::container::{build_container_frame_handler, ContainerFrameHandler};
    use crate::view::map::MapView;
    use crate::view::View;
    use crate::widget::character_stat_line::{build_character_stat_line, CharacterStatLineState};

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