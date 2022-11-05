use std::convert::TryInto;
use std::fmt::format;
use std::io;
use std::io::empty;
use log4rs::config::Logger;
use log::error;

use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use rand_seeder::Seeder;

use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;
use tui::layout::Rect;

use crate::character::{build_player, Character};
use crate::characters::{build_characters, build_default_characters, Characters};
use crate::engine::command::command::Command;
use crate::engine::command::input_mapping;
use crate::engine::command::inventory_command::InventoryCommand;
use crate::engine::command::look_command::LookCommand;
use crate::engine::command::open_command::OpenCommand;
use crate::engine::level::LevelChange::NONE;
use crate::engine::level::{init_level_manager, Level, LevelChange, LevelChangeResult, Levels};

use crate::map::map_generator::{build_dev_inventory};



use crate::map::position::{build_rectangular_area, Position};
use crate::map::position::Side;
use crate::map::room::Room;
use crate::map::tile::Tile::Room as RoomTile;
use crate::menu;
use crate::menu::Selection;

use crate::settings::{Setting, Settings};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui;
use crate::ui::{build_ui, Draw};
use crate::ui::{StartMenuChoice};
use crate::util::utils::UuidEquals;
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::framehandler::character::{CharacterFrameHandler, CharacterFrameHandlerInputResult, ViewMode};
use crate::view::framehandler::character::CharacterFrameHandlerInputResult::VALIDATION;
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::game_over::{build_game_over_menu, GameOver, GameOverChoice};
use crate::view::map::MapView;
use crate::view::settings_menu::SettingsMenu;
use crate::view::util::widget_menu::WidgetMenu;

use crate::widget::character_stat_line::build_character_stat_line;
use crate::widget::boolean_widget::build_boolean_widget;
use crate::widget::text_widget::build_text_input;
use crate::widget::widgets::WidgetList;
use crate::widget::{Widget, WidgetType};

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

    fn print_and_re_render(&mut self, message: String) -> Result<(), io::Error> {
        self.ui.console_print(message);
        self.re_render()
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
        let mut character_view = CharacterFrameHandler { character: base_character.clone(),  widgets: WidgetList { widgets: Vec::new(), selected_widget: None }, view_mode: ViewMode::CREATION};
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
            })?;
            ui.hide_console();

            let key = io::stdin().keys().next().unwrap().unwrap();
            character_creation_result = character_view.handle_input(Some(key))?;

            match character_creation_result.view_specific_result {
                Some(VALIDATION(message)) => {
                    self.ui.console_print(message);
                    self.re_render()?;
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
                    let mut map_view = MapView { map: m, characters: level.characters.clone(), ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, view_area: None };

                    // Adjust the map view size to fit within our borders / make space for the console
                    let map_view_start_pos = Position { x : frame_size.start_position.x + 1, y: frame_size.start_position.y + 1};
                    let map_view_frame_size = Some(build_rectangular_area(map_view_start_pos, frame_size.size_x - 2, frame_size.size_y - 8 ));
                    map_view.draw(map_view_frame_size)?;
                    map_view.draw_containers()?;
                    map_view.draw_characters()?;
                    self.ui.console_print("Arrow keys to move.".to_string());
                    self.re_render()?;
                }
            },
            None => {}
        }
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<(), io::Error> {
        self.terminal_manager.terminal.clear()
    }
}

pub struct GameEngine<B: 'static + tui::backend::Backend>  {
    pub ui_wrapper : UIWrapper<B>,
    settings: Settings,
    levels: Levels,
    game_running : bool,
}

impl <B : Backend> GameEngine<B> {

    pub fn rebuild(&mut self) {
        let settings = build_settings();
        // Grab the randomised seed
        let map_seed = settings.find_string_setting_value("Map RNG Seed".to_string()).unwrap();
        let rng = Seeder::from(map_seed).make_rng();
        self.game_running = false;
        self.levels = init_level_manager(rng);
        self.settings = settings;
    }

    fn handle_settings_menu_selection(&mut self, widgets: WidgetList) -> Result<(), io::Error> {

        for widget in widgets.widgets {
            match widget.state_type {
                WidgetType::Boolean(mut b) => {
                    let setting = self.settings.bool_settings.iter_mut().find(|x| x.name == b.get_name());
                    if let Some(s) = setting {
                        s.value = b.value;
                    }
                },
                WidgetType::Text(mut t) => {
                    let setting = self.settings.string_settings.iter_mut().find(|x| x.name == t.get_name());
                    if let Some(s) = setting {
                        s.value = t.get_input();
                    }
                },
                _ => {}
            }
        }

        // TODO pass this through
        //let fog_of_war = self.settings.find_bool_setting_value(|x| x.name == "Fog of War");

        let map_seed = self.settings.find_string_setting_value( "Map RNG Seed".to_string()).unwrap();
        log::info!("Map seed updated to: {}", map_seed);
        let rng = Seeder::from(map_seed).make_rng();
        self.levels.rng = rng;

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
                let _ui = &mut self.ui_wrapper.ui;
                self.ui_wrapper.draw_start_menu()?;
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

    pub fn start_menu(&mut self, choice: Option<StartMenuChoice>) -> Result<Option<GameOverChoice>, io::Error> {
        loop {
            // Hide additional widgets when paused
            self.ui_wrapper.ui.render_additional = false;
            self.ui_wrapper.draw_start_menu()?;
            let start_choice = if choice.is_some() { choice.clone().unwrap() }  else { self.handle_start_menu_selection()? };
            match start_choice {
                StartMenuChoice::Play => {
                    self.ui_wrapper.ui.render_additional = true;
                    if !self.game_running {
                        log::info!("Starting game..");
                        if let Some(goc) = self.start_game()? {
                            return Ok(Some(goc));
                        }
                        break;
                    } else {
                        return Ok(None);
                    }
                },
                StartMenuChoice::Settings => {
                    log::info!("Showing settings..");

                    let widgets = build_settings_widgets(&self.settings);
                    let mut settings_menu = SettingsMenu {
                        ui: &mut self.ui_wrapper.ui,
                        terminal_manager: &mut self.ui_wrapper.terminal_manager,
                        menu: WidgetMenu {
                            selected_widget: Some(0),
                            widgets: WidgetList { widgets, selected_widget: Some(0) }
                        }
                    };

                    settings_menu.begin()?;
                    let widgets = settings_menu.menu.widgets;

                    self.handle_settings_menu_selection(widgets)?;
                },
                StartMenuChoice::Info => {
                    log::info!("Showing info..");
                    let _ui = &mut self.ui_wrapper.ui;
                    self.ui_wrapper.draw_info()?;
                    io::stdin().keys().next();
                },
                StartMenuChoice::Quit => {
                    if self.game_running {
                        self.game_running = false;
                    }
                    return Ok(Some(GameOverChoice::EXIT));
                }
            }
        }
        Ok(None)
    }

    /*
     * Finds the first entry or exit containing room depending on the direction
     * Sets the player position to match that
     * Returns the room the player has been moved to (for further spawning decisions)
     */
    fn respawn_player(&mut self, change: LevelChange) -> Option<Room> {
        let level = self.levels.get_level_mut();
        let player = level.characters.get_player_mut().unwrap();

        // Grab the first room and set the player's position there
        if let Some(map) = &level.map {
            match change {
                LevelChange::UP => {
                    let exit_room = map.rooms.iter().find(|room| room.get_exit().is_some()).unwrap();
                    player.set_position(exit_room.get_exit().unwrap());
                    return Some(exit_room.clone());
                },
                LevelChange::DOWN => {
                    let entry_room = map.rooms.iter().find(|room| room.get_entry().is_some());
                    if let Some(er) = entry_room {
                        player.set_position(er.get_entry().unwrap());
                        return Some(er.clone());
                    }
                },
                _ => { }
            }
        } else {
            log::error!("Cannot respawn player, Map was None!");
        }
        return None;
    }

    fn respawn_npcs(&mut self, player_room: Room) {
        let level = self.levels.get_level_mut();
        let npcs = level.characters.get_npcs_mut();
        if let Some(map) = &level.map {
            let mut non_player_rooms : Vec<Room> = map.rooms.clone();
            non_player_rooms.retain(|r| !r.uuid_equals(player_room.clone()));

            if !non_player_rooms.is_empty() {
                let mut moved = 0;
                for npc in npcs {
                    // Normal thread RNG / non-reproducible!!
                    let mut rng = thread_rng();
                    let random_room_idx = rng.gen_range(0..non_player_rooms.len() - 1);
                    let chosen_room = non_player_rooms.get(random_room_idx).unwrap();
                    npc.set_position(chosen_room.random_inside_pos(&mut rng));
                }
            } else {
                log::error!("Cannot respawn NPCs, Cannot find any non player containing rooms.");
            }
        } else {
            log::error!("Cannot respawn NPCs, Map was None!");
        }
    }

    fn initialise_characters(&mut self) -> Result<(), io::Error> {
        let player = build_player(String::from("Player"), Position { x: 0, y: 0 });
        let test_npc = build_player(String::from("Rando"), Position { x: 0, y: 0 });
        let mut characters = build_characters(Some(player), vec![test_npc]);
        // Uncomment to use character creation
        //let mut updated_character = self.show_character_creation(characters.get(0).unwrap().clone())?;
        self.levels.get_level_mut().characters = characters;
        let spawn_room = self.respawn_player(LevelChange::DOWN);
        self.respawn_npcs(spawn_room.unwrap().clone());
        self.build_testing_inventory();
        return Ok(());
    }

    pub(crate) fn start_game(&mut self) -> Result<Option<GameOverChoice>, io::Error>{
        self.ui_wrapper.ui.start_menu = menu::build_start_menu(true);
        self.ui_wrapper.print_and_re_render("Generating a new level..".to_string())?;
        self.levels.generate_level();
        self.initialise_characters()?;

        self.game_running = true;
        while self.game_running {
            if self.ui_wrapper.ui.additional_widgets.is_empty() {
                let level = self.levels.get_level_mut();
                let player = level.characters.get_player_mut().unwrap();
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

            let result = self.game_loop()?;
            if result.is_some() {
                return Ok(result);
            }
        }
        //self.terminal_manager.terminal.clear()?;
        Ok(None)
    }

    fn build_testing_inventory(&mut self) {
        let player = self.levels.get_level_mut().characters.get_player_mut().unwrap();
        player.set_inventory(build_dev_inventory());
    }

    fn handle_player_movement(&mut self, side: Side) -> Result<Option<GameOverChoice>, io::Error> {
        let levels = &mut self.levels;
        let level = levels.get_level_mut();
        let updated_position = level.find_player_side_position(side).clone();

        let mut level_change = NONE;
        match updated_position {
            Some(pos) => {
                if let Some(m) = &level.map {
                    if m.is_traversable(pos) {
                        let player =  level.characters.get_player_mut().unwrap();
                        player.set_position(pos);
                    }

                    if let Some(room) = m.rooms.iter()
                        .find(|r| r.get_inside_area().contains_position(pos)) {
                        if pos.equals_option(room.get_exit()) {
                            self.ui_wrapper.print_and_re_render("You've reached the exit! You move down a level..".to_string())?;
                            io::stdin().keys().next().unwrap().unwrap();
                            level_change = LevelChange::DOWN;
                        } else if pos.equals_option(room.get_entry()) {
                            self.ui_wrapper.print_and_re_render("You've reached the entry! You move up a level..".to_string())?;
                            io::stdin().keys().next().unwrap().unwrap();
                            level_change = LevelChange::UP;
                        }
                    }
                }

                match level_change {
                    NONE => {},
                    change => {
                        let changed = levels.change_level(change.clone());
                        if changed.is_ok() {
                            match changed.unwrap() {
                                LevelChangeResult::LevelChanged => {
                                    self.respawn_player(change);
                                },
                                LevelChangeResult::OutOfDungeon => {
                                    let player_score = self.levels.get_level_mut().characters.get_player_mut().unwrap().get_inventory_mut().get_loot_value();

                                    let mut menu = build_game_over_menu(
                                        format!("You left the dungeon.\nLoot Total: {}", player_score),
                                        &mut self.ui_wrapper.ui,
                                        &mut self.ui_wrapper.terminal_manager);
                                    let result = menu.begin()?;
                                    if let Some(game_over_choice) = result.view_specific_result {
                                        return Ok(Some(game_over_choice));
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            },
            None => {}
        }
        Ok(None)
    }

    pub fn menu_command(&mut self) -> Result<Option<GameOverChoice>, io::Error> {
        self.ui_wrapper.clear_screen()?;
        self.ui_wrapper.ui.hide_console();
        if let Some(goc) = self.start_menu(None)? {
            self.ui_wrapper.ui.show_console();
            self.ui_wrapper.clear_screen()?;
            return Ok(Some(goc));
        }
        self.ui_wrapper.ui.show_console();
        self.ui_wrapper.clear_screen()?;
        Ok(None)
    }

    pub fn handle_input(&mut self, key : Key) -> Result<Option<GameOverChoice>, io::Error>  {
        let level = self.levels.get_level_mut();
        match key {
            Key::Char('q') => {
                if let Some(goc) = self.menu_command()? {
                    return Ok(Some(goc));
                }
            },
            Key::Char('i') => {
                let mut command = InventoryCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
                command.handle(key)?;
            },
            Key::Char('k') => {
                let mut command = LookCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
                command.handle(key)?;
            },
            Key::Char('o') => {
                self.ui_wrapper.print_and_re_render("What do you want to open?. Arrow keys to choose. Repeat command to choose current location.".to_string())?;
                let mut command = OpenCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
                command.handle(key)?;
            },
            Key::Down | Key::Up | Key::Left | Key::Right => {
                if let Some(side) = input_mapping::key_to_side(key) {
                    if let Some(game_over_choice) = self.handle_player_movement(side)? {
                        return Ok(Some(game_over_choice));
                    }
                }
            },
            _ => {}
        }
        Ok(None)
    }

    fn game_loop(&mut self) -> Result<Option<GameOverChoice>, io::Error> {
        let key = io::stdin().keys().next().unwrap().unwrap();
        //self.terminal_manager.terminal.clear()?;
        return Ok(self.handle_input(key)?);
    }
}

pub fn build_settings() -> Settings {
    let fog_of_war : Setting<bool> = Setting { name: "Fog of War".to_string(), value: false };

    let random_seed: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    let map_seed : Setting<String> = Setting { name: "Map RNG Seed".to_string(), value: random_seed };
    Settings { bool_settings: vec![fog_of_war], string_settings: vec![map_seed]}
}

pub fn build_settings_widgets(settings : &Settings) -> Vec<Widget> {
    let mut widgets = Vec::new();
    for setting in &settings.bool_settings {
        widgets.push(build_boolean_widget(15, setting.name.clone(), setting.value))
    }
    for setting in &settings.string_settings {
        widgets.push(build_text_input(15, setting.name.clone(), setting.value.clone(), 1))
    }
    widgets
}

pub fn build_game_engine<'a, B: tui::backend::Backend>(terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, io::Error> {
    let ui = build_ui();
    let settings = build_settings();
    // Grab the randomised seed
    let map_seed = settings.find_string_setting_value("Map RNG Seed".to_string()).unwrap();
    let rng = Seeder::from(map_seed).make_rng();
    Ok(GameEngine { levels: init_level_manager(rng), settings, ui_wrapper : UIWrapper { ui, terminal_manager }, game_running: false})
}

pub fn build_test_game_engine<'a, B: tui::backend::Backend>(levels: Levels, terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, io::Error> {
    let ui = build_ui();
    let settings = build_settings();
    Ok(GameEngine { levels, settings, ui_wrapper : UIWrapper { ui, terminal_manager }, game_running: false})
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use termion::event::Key;

    use crate::character::{build_player, Character};
    use crate::characters::build_empty_characters;
    use crate::engine::game_engine::*;
    use crate::map::{Map, Tiles};
    use crate::map::position::{Position};
    use crate::map::position::{Area, build_square_area};
    use crate::map::tile::{build_library, Tile, TileDetails};
    use crate::terminal::terminal_manager;
    use crate::view::View;

    fn build_tiles(map_area: Area, tile : Tile) -> Vec<Vec<TileDetails>> {
        let tile_library = build_library();
        let mut map_tiles = Vec::new();
        let mut row;
        for _y in map_area.start_position.y..=map_area.end_position.y {
            row = Vec::new();
            for _x in map_area.start_position.x..=map_area.end_position.x {
                row.push( tile_library[&tile].clone());
            }
            map_tiles.push(row);
        }
        map_tiles
    }

    fn test_movement_input(levels: Levels, start_position: Position, input: Vec<Key>, expected_end_position : Position) {
        // GIVEN a game engine with a 3x3 grid of tiles
        let _tile_library = build_library();
        let terminal_manager = terminal_manager::init_test().unwrap();
        let game_engine = build_test_game_engine(levels, terminal_manager);

        match game_engine {
            Result::Ok(mut engine) => {
                {
                    let characters = build_empty_characters();
                    let level = &mut engine.levels.get_level_mut();
                    level.characters = characters;
                }

                // AND The player is placed in the middle of the map
                assert_eq!(start_position, engine.levels.get_level_mut().characters.get_player().unwrap().get_position());

                // WHEN we push the down key
                for key in input {
                    engine.handle_input(key).unwrap();
                }

                // THEN we expect the player to be moved into the traversable tile
                assert_eq!(expected_end_position, engine.levels.get_level_mut().characters.get_player().unwrap().get_position());
            },
            _ => {
                panic!("Expected a valid Game Engine instance!")
            }
        }
    }

    fn build_test_levels(map: Map) -> Levels {
        let level = Level {
            map: Some(map.clone()),
            characters: build_empty_characters()
        };

        let rng = Seeder::from("test".to_string()).make_rng();
        let mut levels = init_level_manager(rng);
        levels.add_level(level);
        levels
    }

    #[test]
    fn test_build_game_engine() {
        let terminal_manager = terminal_manager::init_test().unwrap();
        let _game_engine = build_game_engine(terminal_manager);
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
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move down
        let input = vec![Key::Down];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:1, y: 2};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move down
        let input = vec![Key::Down];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the bottom middle of the map
        let start_position = Position{x:1, y:2};
        // WHEN we attempt to move up
        let input = vec![Key::Up];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle end of the map
        let start_position = Position{x:1, y:2};
        // WHEN we attempt to move up
        let input = vec![Key::Up];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 2};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move left
        let input = vec![Key::Left];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:0, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move left
        let input = vec![Key::Left];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:2, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
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
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        let levels = build_test_levels(map);

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
    }

}