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
use crate::engine::game_engine::LevelChange::NONE;
use crate::engine::level::{Characters, Level};
use crate::map::Map;
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
use crate::view::framehandler::character::{CharacterFrameHandler, CharacterFrameHandlerInputResult, ViewMode};
use crate::view::framehandler::character::CharacterFrameHandlerInputResult::VALIDATION;
use crate::view::map::MapView;
use crate::widget::character_stat_line::build_character_stat_line;

pub struct Levels {
    // Implied to always reflect updates to levels
    _current_level: usize,
    levels : Vec<Level>
}

#[derive(Clone)]
enum LevelChange {
    UP,
    DOWN,
    NONE
}

pub struct UIWrapper<B: 'static + tui::backend::Backend> {
    ui : ui::UI,
    terminal_manager : TerminalManager<B>,
}

impl <B : Backend> UIWrapper<B> {
    // TODO refactor into a singular component shared with commands
    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn print_and_re_render(&mut self, message: String) {
        self.ui.console_print(message);
        self.re_render();
    }

    fn draw_settings_menu(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) })
    }

    fn draw_start_menu(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })
    }

    fn draw_info(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_info(frame) })
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
                    self.ui.console_print(message);
                    self.re_render();
                },
                Some(CharacterFrameHandlerInputResult::NONE) => {
                    return Ok(character_view.get_character())
                },
                _ => {}
            }
        }
        return Ok(character_view.get_character());
    }

    fn draw_map_view(&mut self, level: &mut Level) -> Result<(), io::Error> {
        let map = &mut level.get_map_mut().cloned();
        let frame_size_copy = self.ui.frame_size.clone();
        match map {
            Some(m) => {
                if let Some(frame_size) = frame_size_copy {
                    let mut map_view = MapView { map: m, characters: level.characters.characters.clone(), ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, view_area: None };

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

    fn clear_console(&mut self) -> Result<(), io::Error> {
        self.terminal_manager.terminal.clear()
    }
}

impl Levels {
    fn add_level(&mut self, level: Level) {
        self.levels.push(level);
    }

    fn get_level_mut(&mut self) -> &mut Level {
        return self.levels.get_mut(self._current_level).unwrap();
    }

    fn generate_level(&mut self) {
        let map_area = build_rectangular_area(Position { x: 0, y: 0 }, 20, 20);
        let mut map_generator = build_generator(map_area);

        let mut new_level;
        if !self.levels.is_empty() {
            let player = self.get_level_mut().characters.remove_player();
            new_level = Level {
                map: Some(map_generator.generate()),
                characters: Characters { characters: vec![player] }
            };
        } else {
            new_level = Level {
                map: Some(map_generator.generate()),
                characters: Characters { characters: Vec::new() }
            };
        }

        self.levels.push(new_level);
    }

    fn change_level(&mut self, level_change: LevelChange) -> Result<bool, io::Error>  {
        match level_change {
            LevelChange::UP => {
                if self._current_level > 0 {
                    let player = self.get_level_mut().characters.remove_player();
                    self._current_level -= 1;
                    self.get_level_mut().characters.set_characters(vec![player]);
                    return Ok(true);
                }
            },
            LevelChange::DOWN => {
                if self._current_level < self.levels.len() - 1 {
                    let player = self.get_level_mut().characters.remove_player();
                    self._current_level += 1;
                    self.get_level_mut().characters.set_characters(vec![player]);
                } else {
                    self.generate_level();
                    self._current_level += 1;
                }
                return Ok(true);
            },
            _ => {
            }
        }
        return Ok(false);
    }
}


pub struct GameEngine<B: 'static + tui::backend::Backend>  {
    ui_wrapper : UIWrapper<B>,
    levels: Levels,
    settings : settings::EnumSettings,
    game_running : bool,
}

impl <B : Backend> GameEngine<B> {

    fn handle_settings_menu_selection(&mut self) -> Result<(), io::Error> {
        loop {
            let last_selection = self.ui_wrapper.ui.settings_menu.selection;
            let key = io::stdin().keys().next().unwrap().unwrap();
            self.ui_wrapper.ui.settings_menu.handle_input(key);
            let selection = self.ui_wrapper.ui.settings_menu.selection;
            log::info!("Selection is: {}", selection);
            if last_selection != selection {
                log::info!("Selection changed to: {}", selection);
                self.ui_wrapper.draw_settings_menu()?;
            }

            if self.ui_wrapper.ui.settings_menu.exit {
                log::info!("Menu exited.");
                break;
            }

            if self.ui_wrapper.ui.settings_menu.selected {
                match self.ui_wrapper.ui.settings_menu.selection.try_into() {
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
            let last_selection = self.ui_wrapper.ui.start_menu.selection;
            let key = io::stdin().keys().next().unwrap().unwrap();
            self.ui_wrapper.ui.start_menu.handle_input(key);
            let selection = self.ui_wrapper.ui.start_menu.selection;
            log::info!("Selection is: {}", selection);
            if last_selection != selection {
                log::info!("Selection changed to: {}", selection);
                let ui = &mut self.ui_wrapper.ui;
                self.ui_wrapper.draw_start_menu();
            }

            if self.ui_wrapper.ui.start_menu.exit {
                log::info!("Menu exited.");
                return Ok(StartMenuChoice::Quit);
            }

            if self.ui_wrapper.ui.start_menu.selected {
                match self.ui_wrapper.ui.start_menu.selection.try_into() {
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
            self.ui_wrapper.ui.render_additional = false;
            self.ui_wrapper.draw_start_menu()?;
            let start_choice = self.handle_start_menu_selection()?;
            match start_choice {
                StartMenuChoice::Play => {
                    self.ui_wrapper.ui.render_additional = true;
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
                    let ui = &mut self.ui_wrapper.ui;
                    self.ui_wrapper.draw_settings_menu();
                    self.handle_settings_menu_selection()?;
                },
                StartMenuChoice::Info => {
                    log::info!("Showing info..");
                    let ui = &mut self.ui_wrapper.ui;
                    self.ui_wrapper.draw_info();
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

    fn respawn_player(&mut self, change: LevelChange) {
        let level = self.levels.get_level_mut();
        let mut player = level.characters.get_player_mut();

        // Grab the first room and set the player's position there
        if let Some(map) = &level.map {
            match change {
                LevelChange::UP => {
                    let exit_room = map.rooms.iter().find(|room| room.exit.is_some()).unwrap();
                    player.set_position(exit_room.exit.unwrap());
                },
                LevelChange::DOWN => {
                    let entry_room = map.rooms.iter().find(|room| room.entry.is_some()).unwrap();
                    player.set_position(entry_room.entry.unwrap());
                },
                _ => { }
            }
        }
    }

    fn initialise_characters(&mut self) -> Result<(), io::Error> {
        let mut characters = self.build_characters();
        // Uncomment to use character creation
        //let mut updated_character = self.show_character_creation(characters.get(0).unwrap().clone())?;
        self.levels.get_level_mut().characters = Characters { characters: characters.clone() };
        self.respawn_player(LevelChange::DOWN);
        self.build_testing_inventory();
        return Ok(());
    }

    fn start_game(&mut self) -> Result<(), io::Error>{
        self.ui_wrapper.ui.start_menu = menu::build_start_menu(true);
        self.ui_wrapper.print_and_re_render("Generating a new level..".to_string());
        self.levels.generate_level();
        self.initialise_characters()?;

        self.game_running = true;
        while self.game_running {
            if self.ui_wrapper.ui.additional_widgets.is_empty() {
                let level = self.levels.get_level_mut();
                let player = level.characters.get_player_mut();
                let stat_line = build_character_stat_line(player.get_health(), player.get_details(), player.get_inventory_mut().get_loot_value());
                self.ui_wrapper.ui.additional_widgets.push(stat_line);
            }

            self.ui_wrapper.ui.show_console();

            let level = self.levels.get_level_mut();
            match self.ui_wrapper.draw_map_view(level) {
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
        self.levels.get_level_mut().characters.characters[0].set_inventory(build_dev_inventory());
    }

    fn handle_player_movement(&mut self, side: Side) {
        let levels = &mut self.levels;
        let level = levels.get_level_mut();
        let updated_position = level.find_player_side_position(side).clone();

        let mut level_change = NONE;
        match updated_position {
            Some(pos) => {
                if let Some(m) = &level.map {
                    if m.is_traversable(pos) {
                        let player =  level.characters.get_player_mut();
                        player.set_position(pos);
                    }

                    if let Some(room) = m.rooms.iter()
                        .find(|r| r.get_inside_area().contains_position(pos)) {
                        if pos.equals_option(room.exit) {
                            self.ui_wrapper.print_and_re_render("You've reached the exit! You move down a level..".to_string());
                            io::stdin().keys().next().unwrap().unwrap();
                            level_change = LevelChange::DOWN;
                        } else if pos.equals_option(room.entry) {
                            self.ui_wrapper.print_and_re_render("You've reached the entry! You move up a level..".to_string());
                            io::stdin().keys().next().unwrap().unwrap();
                            level_change = LevelChange::UP;
                        }
                    }
                }

                match level_change {
                    NONE => {},
                    change => {
                        let changed = levels.change_level(change.clone());
                        if changed.is_ok() && changed.unwrap()  {
                            self.respawn_player(change);
                        }
                    }
                }
            },
            None => {}
        }
    }

    pub fn menu_command(&mut self) -> Result<(), io::Error> {
        self.ui_wrapper.clear_console();
        self.ui_wrapper.ui.hide_console();
        self.start_menu()?;
        self.ui_wrapper.ui.show_console();
        self.ui_wrapper.clear_console();
        Ok(())
    }

    pub fn handle_input(&mut self, key : Key) -> Result<(), io::Error>  {
        let level = self.levels.get_level_mut();
        match key {
            Key::Char('q') => {
                self.menu_command();
            },
            Key::Char('i') => {
                let mut command = InventoryCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
                command.handle(key);
            },
            Key::Char('k') => {
                let mut command = LookCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
                command.handle(key);
            },
            Key::Char('o') => {
                self.ui_wrapper.print_and_re_render("What do you want to open?. Arrow keys to choose. Repeat command to choose current location.".to_string());
                let mut command = OpenCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
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

pub fn init_level_manager() -> Levels {
    Levels { levels: vec![], _current_level: 0}
}

pub fn build_game_engine<'a, B: tui::backend::Backend>(mut terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, io::Error> {
    let ui = build_ui();
    let fog_of_war = settings::Setting { name : "Fog of war".to_string(), value : false };
    let settings = settings::EnumSettings { settings: vec![fog_of_war] };
    Ok(GameEngine { levels: init_level_manager(), ui_wrapper : UIWrapper { ui, terminal_manager }, settings, game_running: false})
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
                {
                    let characters = build_characters(start_position);
                    let level = &mut engine.levels.get_level_mut();
                    level.characters.set_characters(characters);
                    level.set_map(Some(map));
                }

                // AND The player is placed in the middle of the map
                assert_eq!(start_position, engine.levels.get_level_mut().characters.get_player().get_position());

                // WHEN we push the down key
                for key in input {
                    engine.handle_input(key);
                }

                // THEN we expect the player to be moved into the traversable tile
                assert_eq!(expected_end_position, engine.levels.get_level_mut().characters.get_player().get_position());
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