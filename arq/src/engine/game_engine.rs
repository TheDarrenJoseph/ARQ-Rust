use std::convert::TryInto;
use std::fmt::format;
use std::{io, thread};
use std::io::empty;
use std::time::Duration;
use log4rs::config::Logger;
use log::{error, log};

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
use crate::error_utils::error_result;

use crate::map::map_generator::{build_dev_inventory};

use crate::map::position::{build_rectangular_area, Position};
use crate::map::position::Side;
use crate::map::room::Room;
use crate::map::tile::Tile::Room as RoomTile;
use crate::{menu, sound};
use crate::menu::Selection;

use crate::settings::{build_settings, Setting, SETTING_BG_MUSIC, SETTING_FOG_OF_WAR, SETTING_RNG_SEED, Settings};
use crate::sound::sound::{build_sound_sinks, SoundSinks};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{build_ui, get_input_key, StartMenuChoice};
use crate::ui::ui_wrapper::UIWrapper;
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
use crate::widget::widgets::{build_settings_widgets, WidgetList};
use crate::widget::{Widget, WidgetType};


pub struct GameEngine<B: 'static + tui::backend::Backend>  {
    ui_wrapper : UIWrapper<B>,
    settings: Settings,
    levels: Levels,
    sound_sinks: Option<SoundSinks>,
    game_running : bool,
}

impl <B : Backend> GameEngine<B> {

    pub fn rebuild(&mut self) {
        let settings = build_settings();
        // Grab the randomised seed
        let map_seed = settings.find_string_setting_value(SETTING_RNG_SEED.to_string()).unwrap();

        let rng = Seeder::from(map_seed).make_rng();
        self.game_running = false;
        self.levels = init_level_manager(rng);
        self.settings = settings;
    }

    // Saves the widget values into the settings
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
                WidgetType::Number(mut t) => {
                    let setting = self.settings.u32_settings.iter_mut().find(|x| x.name == t.get_name());
                    if let Some(s) = setting {
                        s.value = t.get_input() as u32;
                    }
                },
                _ => {}
            }
        }

        Ok(())
    }

    // Updates the game to reflect current settings
    fn update_from_settings(&mut self) -> Result<(), io::Error>  {
        // TODO pass this through
        let fog_of_war = self.settings.find_bool_setting_value(SETTING_FOG_OF_WAR.to_string()).unwrap();

        let map_seed = self.settings.find_string_setting_value( SETTING_RNG_SEED.to_string()).unwrap();
        log::info!("Map seed updated to: {}", map_seed);
        let rng = Seeder::from(map_seed).make_rng();
        self.levels.rng = rng;

        let bg_music_volume = self.settings.find_u32_setting_value(SETTING_BG_MUSIC.to_string()).unwrap();
        if let Some(sinks) = &mut self.sound_sinks {
            sinks.get_bg_sink_mut().configure(bg_music_volume);
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

    fn setup_sinks(&mut self) -> Result<(), io::Error> {
        if self.sound_sinks.is_none() {
            // Start the sound sinks and play bg music
            let mut sinks = build_sound_sinks();
            let bg_music_volume = self.settings.find_u32_setting_value(SETTING_BG_MUSIC.to_string()).unwrap();
            sinks.get_bg_sink_mut().configure(bg_music_volume);
            sinks.get_bg_sink_mut().play();
            self.sound_sinks = Some(sinks);
        }
        Ok(())
    }

    pub fn start_menu(&mut self, choice: Option<StartMenuChoice>) -> Result<Option<GameOverChoice>, io::Error> {
        self.setup_sinks();
        loop {
            // Hide additional widgets when paused
            self.ui_wrapper.ui.render_additional = false;
            self.ui_wrapper.draw_start_menu()?;
            let start_choice = if choice.is_some() { choice.clone().unwrap() }  else { self.handle_start_menu_selection()? };
            self.ui_wrapper.clear_screen();
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
                    // Ensure we're using any changes to the settings
                    self.update_from_settings();
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
        if let Some(sr) = spawn_room {
            self.respawn_npcs(sr.clone());
            self.build_testing_inventory();
            return Ok(());
        } else {
            return error_result(String::from("Player was not properly spawned/no player spawn room returned!"));
        }
    }

    fn initialise(&mut self) -> Result<(), io::Error> {
        self.ui_wrapper.ui.start_menu = menu::build_start_menu(true);
        self.ui_wrapper.print_and_re_render(String::from("Generating a new level.."))?;

        let mut generated = false;
        while !generated {
            self.levels.generate_level();
            match self.initialise_characters() {
                Err(e) => {
                    log::error!("Generation round failed with error {}. Trying again...", e);
                    self.ui_wrapper.print_and_re_render(String::from("Bad level. Trying again..."));
                },
                Ok(_) => {
                    generated = true;
                },
            }
        }
        Ok(())
    }

    fn add_additional_widgets(&mut self) {
        if self.ui_wrapper.ui.additional_widgets.is_empty() {
            let level = self.levels.get_level_mut();
            let player = level.characters.get_player_mut().unwrap();
            let stat_line = build_character_stat_line(player.get_health(), player.get_details(), player.get_inventory_mut().get_loot_value());
            self.ui_wrapper.ui.additional_widgets.push(stat_line);
        }
    }

    pub(crate) fn start_game(&mut self) -> Result<Option<GameOverChoice>, io::Error>{
        self.initialise()?;
        self.game_running = true;
        while self.game_running {
            self.add_additional_widgets();
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
                            match self.ui_wrapper.yes_or_no(
                                String::from("You've reached the exit! There's a staircase downwards; would you like to leave?"),
                                Some(String::from("You move downstairs a level.."))) {
                                Ok(true) => {
                                    level_change = LevelChange::DOWN;
                                },
                                _ => {}
                            }
                        } else if pos.equals_option(room.get_entry()) {
                            match self.ui_wrapper.yes_or_no(
                                String::from("This is the entrance. There's a staircase upwards; wold you like to leave?"),
                                Some(String::from("You move upstairs a level.."))) {
                                Ok(true) => {
                                    level_change = LevelChange::UP;
                                },
                                _ => {}
                            }
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
                let key = self.ui_wrapper.get_prompted_input(String::from("What do you want to open?. Arrow keys to choose. Repeat command to choose current location."))?;
                let mut command = OpenCommand { level, ui: &mut self.ui_wrapper.ui, terminal_manager: &mut self.ui_wrapper.terminal_manager };
                command.handle(key)?;
            },
            Key::Down | Key::Up | Key::Left | Key::Right | Key::Char('w') | Key::Char('a') | Key::Char('s') | Key::Char('d') => {
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

    fn player_turn(&mut self)  -> Result<Option<GameOverChoice>, io::Error> {
        let key = get_input_key()?;
        //self.terminal_manager.terminal.clear()?;
        return Ok(self.handle_input(key)?);
    }

    fn npc_turns(&mut self)  -> Result<(), io::Error> {
        // TODO NPC movement
        return Ok(());
    }

    fn game_loop(&mut self) -> Result<Option<GameOverChoice>, io::Error> {
        let game_over_choice = self.player_turn()?;
        self.npc_turns()?;
        return Ok(game_over_choice);
    }
}

pub fn build_game_engine<'a, B: tui::backend::Backend>(terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, io::Error> {
    let ui = build_ui();
    let settings = build_settings();
    // Grab the randomised seed
    let map_seed = settings.find_string_setting_value(SETTING_RNG_SEED.to_string()).unwrap();
    let rng = Seeder::from(map_seed).make_rng();
    Ok(GameEngine { levels: init_level_manager(rng), settings, ui_wrapper : UIWrapper { ui, terminal_manager }, sound_sinks: None, game_running: false})
}

pub fn build_test_game_engine<'a, B: tui::backend::Backend>(levels: Levels, terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, io::Error> {
    let ui = build_ui();
    let settings = build_settings();
    Ok(GameEngine { levels, settings, ui_wrapper : UIWrapper { ui, terminal_manager }, sound_sinks: None, game_running: false})
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
                let levels = engine.levels.get_level_mut();
                let player = levels.characters.get_player_mut();
                assert_eq!(start_position, player.unwrap().get_position());

                for key in input {
                    engine.handle_input(key).unwrap();
                }
                assert_eq!(expected_end_position, engine.levels.get_level_mut().characters.get_player().unwrap().get_position());
            },
            _ => {
                panic!("Expected a valid Game Engine instance!")
            }
        }
    }

    fn build_test_levels(map: Map, player: Character) -> Levels {
        let level = Level {
            map: Some(map.clone()),
            characters: build_characters(Some(player), Vec::new())
        };

        let rng = Seeder::from("test".to_string()).make_rng();
        let mut levels = init_level_manager(rng);
        levels.add_level(level);
        levels
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
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

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

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);
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

        // AND the player start position is the bottom middle of the map
        let start_position = Position{x:1, y:2};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

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

        // AND the player start position is the middle end of the map
        let start_position = Position{x:1, y:2};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

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

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

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

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

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

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

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

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player = build_player(String::from("Test Player"), start_position);
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
    }

}