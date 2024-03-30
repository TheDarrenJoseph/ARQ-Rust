use std::convert::TryInto;

use std::{io};

use std::collections::HashMap;
use std::future::Future;
use std::io::{Error, ErrorKind};
use std::pin::Pin;





use log::{error, info};


use rand::{Rng, thread_rng};
use rand_seeder::Seeder;

use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;




use futures::FutureExt;


use crate::character::characters::Characters;
use crate::engine::command::command::Command;
use crate::engine::command::input_mapping;
use crate::engine::command::inventory_command::InventoryCommand;
use crate::engine::command::look_command::LookCommand;
use crate::engine::command::open_command::OpenCommand;

use crate::engine::level::{init_level_manager, LevelChange, LevelChangeResult, Levels};


use crate::map::position::{Area, Side};
use crate::map::room::Room;

use crate::{menu, widget};
use crate::character::battle::Battle;
use crate::character::builder::character_builder::{build_dev_player_inventory, CharacterBuilder, CharacterPattern};
use crate::engine::combat::Combat;
use crate::engine::input_handler::handle_input;
use crate::engine::menu::start_menu;
use crate::map::Map;
use crate::menu::Selection;


use crate::engine::process::map_generation::MapGeneration;
use crate::error::io_error_utils::error_result;


use crate::settings::{build_settings, SETTING_BG_MUSIC, SETTING_RESOLUTION, SETTING_RNG_SEED, Settings};
use crate::sound::sound::{build_sound_sinks, SoundSinks};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::resolution::Resolution;
use crate::ui::ui::{build_ui, get_input_key, StartMenuChoice, UI};
use crate::ui::ui_areas_builder::UIAreasBuilder;
use crate::ui::ui_layout::LayoutType::SINGLE_MAIN_WINDOW_CENTERED;
use crate::ui::ui_wrapper::UIWrapper;
use crate::util::utils::UuidEquals;
use crate::view::{InputHandler, View};
use crate::view::combat_view::CombatView;
use crate::view::dialog_view::DialogView;




use crate::view::game_over_view::{build_game_over_menu, GameOverChoice};
use crate::view::settings_menu_view::SettingsMenuView;
use crate::view::model::usage_line::{UsageCommand, UsageLine};
use crate::view::util::widget_menu::WidgetMenu;



use crate::widget::widgets::{build_settings_widgets, WidgetList};
use crate::widget::{StandardWidgetType, StatefulWidgetType};
use crate::view::framehandler::map_generation::MapGenerationFrameHandler;
use crate::view::util::callback::Callback;
use crate::view::util::progress_display::ProgressDisplay;
use crate::view::util::callback::CallbackHandler;
use crate::widget::character_stat_line::CharacterStatLineWidget;
use crate::widget::stateful::dropdown_widget::get_resolution_dropdown_options;

pub struct GameEngine<B: 'static + Backend>  {
    pub ui_wrapper : UIWrapper<B>,
    pub(crate) settings: Settings,
    pub levels: Levels,
    sound_sinks: Option<SoundSinks>,
    game_running : bool,
}

impl <B : Backend + Send> GameEngine<B> {
    pub fn is_game_running(&self) -> bool {
        self.game_running
    }

    pub fn stop_game(&mut self) {
        self.game_running = false
    }

    pub fn rebuild(&mut self) {
        let settings = build_settings();
        // Grab the randomised seed
        let map_seed = settings.find_string_setting_value(SETTING_RNG_SEED.to_string()).unwrap();
        let seed_copy = map_seed.clone();
        let rng = Seeder::from(map_seed).make_rng();
        self.game_running = false;
        self.levels = init_level_manager(seed_copy, rng);
        self.settings = settings;
    }


    // Updates the game to reflect current settings
    pub(crate) fn update_from_settings(&mut self) -> Result<(), Error>  {
        let _fog_of_war = self.settings.is_fog_of_war();
        let map_seed = self.settings.get_rng_seed().ok_or( Error::new(ErrorKind::NotFound, "Failed to retrieve map seed"))?;
        info!("Map seed updated to: {}", map_seed);
        let rng = Seeder::from(map_seed).make_rng();
        self.levels.rng = rng;

        let bg_music_volume = self.settings.get_bg_music_volume();
        if let Some(sinks) = &mut self.sound_sinks {
            sinks.get_bg_sink_mut().configure(bg_music_volume);
        }

        let resolution = self.settings.get_resolution();
        if let Some(res) = resolution.value {
            info!("Re-init UI with resolution: {:?}", res);
            self.ui_wrapper.ui.re_init::<B>(Area::from_resolution(res));
        } else {
            // Get the current terminal size and use that for the fullscreen size
            let resolution = Resolution::from_rect(self.ui_wrapper.terminal_manager.terminal.get_frame().size());
            info!("Re-init UI with fullscreen resolution: {:?}", resolution);
            self.ui_wrapper.ui.re_init::<B>(Area::from_resolution(resolution));
        }
        Ok(())
    }

    fn setup_sinks(&mut self) -> Result<(), Error> {
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

    pub fn init(&mut self) -> Result<(), Error> {
        self.setup_sinks()?;

        let ui_wrapper = &mut self.ui_wrapper;
        let ui = &mut ui_wrapper.ui;
        let terminal_manager = &mut ui_wrapper.terminal_manager;

        let resolution_option = self.settings.find_dropdown_setting_value(SETTING_RESOLUTION.to_string()).unwrap();

        let resolution_value = resolution_option.value.clone();
        terminal_manager.terminal.draw(|frame| {
            let mut frame_area: Area;
            if resolution_option.display_name == "FULLSCREEN" {
                info!("FULLSCREEN resolution selected. Using frame size: {:?}", frame.size());
                frame_area = Area::from_rect(frame.size());
            } else {
                let res_value = resolution_value.unwrap();
                info!("resolution selected. Using resolution: {:?}", res_value);
                frame_area = Area::from_resolution(res_value)
            }
            ui.init::<B>(frame_area);
        })?;

        let ui_layout = &mut ui.ui_layout;
        let ui_layout = ui_layout.as_mut().ok_or("Failed to get ui_layout, has it been initialised?").unwrap();
        terminal_manager.terminal.draw(|frame| {
            ui_layout.init_areas(frame.size());
        })?;

        Ok(())
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
                let _moved = 0;
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

    fn initialise_characters(&mut self) -> Result<(), Error> {
        info!("Building player...");
        let player_pattern_result = CharacterPattern::new_player();
        if player_pattern_result.is_err() {
            return Err(player_pattern_result.unwrap_err().to_io_error())
        }
        let player = CharacterBuilder::new(player_pattern_result.unwrap()).build(String::from("Player"));
        info!("Building NPCs...");
        let npc_pattern_result = CharacterPattern::goblin();
        if npc_pattern_result.is_err() {
            return Err(npc_pattern_result.unwrap_err().to_io_error())
        }
        let test_npc = CharacterBuilder::new(npc_pattern_result.unwrap()).build(String::from("Ruggo"));

        let characters = Characters::new(Some(player), vec![test_npc]);
        // Uncomment to use character creation
        //let mut updated_character = self.show_character_creation(characters.get(0).unwrap().clone())?;
        self.levels.get_level_mut().characters = characters;
        let spawn_room = self.respawn_player(LevelChange::DOWN);
        return if let Some(sr) = spawn_room {
            self.respawn_npcs(sr.clone());
            self.build_testing_inventory();
            Ok(())
        } else {
            error_result(String::from("Player was not properly spawned/no player spawn room returned!"))
        }
    }

    async fn generate_map(&mut self) -> Result<Map, Error> {
        let seed = self.levels.get_seed();
        let map_framehandler = MapGenerationFrameHandler { seed: seed.clone() };

        let map_generator = self.levels.build_map_generator();
        let size_x = map_generator.map.area.width;
        let size_y = map_generator.map.area.height;

        let progress_display = ProgressDisplay {
            terminal_manager: &mut self.ui_wrapper.terminal_manager,
            frame_handler: map_framehandler
        };
        let mut level_generator = MapGeneration {
            map_generator,
            progress_display
        };

        info!("Generating map using RNG seed: {} and size: {}, {}", seed, size_x, size_y);
        level_generator.generate_level().await
    }

    async fn initialise(&mut self) -> Result<(), Error> {
        self.ui_wrapper.print_and_re_render(String::from("Generating a new level.."))?;

        let map = self.generate_map().await.unwrap();
        self.levels.add_level(map);
        return match self.initialise_characters() {
            Err(e) => {
                Err(e)
            },
            Ok(_) => {
                info!("Map generated successfully");
                Ok(())
            },
        }
    }

    fn add_or_update_additional_widgets(&mut self) {
        let additional_widgets = self.ui_wrapper.ui.get_additional_widgets();
        if additional_widgets.is_empty() {
            let level_number = self.levels.get_current_level() as i32 + 1;
            let level = self.levels.get_level_mut();
            let player = level.characters.get_player_mut().unwrap();
            let stat_line = CharacterStatLineWidget::new(
                level_number,
                player.get_health(),
                player.get_details(),
                player.get_inventory_mut().get_loot_value());
            self.ui_wrapper.ui.get_additional_widgets_mut().push(StandardWidgetType::StatLine(stat_line));

            let mut commands : HashMap<Key, UsageCommand> = HashMap::new();
            commands.insert(Key::Char('i'), UsageCommand::new('i', String::from("Inventory/Info") ));
            let map_usage_line = UsageLine::new(commands);
            self.ui_wrapper.ui.get_additional_widgets_mut().push(StandardWidgetType::UsageLine(map_usage_line));

        } else {
            let widgets_mut = self.ui_wrapper.ui.get_additional_widgets_mut();
            match widgets_mut.get_mut(0) {
                Some(StandardWidgetType::StatLine(s)) => {
                    let level_number = self.levels.get_current_level() as i32 + 1;
                    let level = self.levels.get_level_mut();
                    let player = level.characters.get_player_mut().unwrap();
                    s.set_health(player.get_health());
                    s.set_level(level_number);
                    s.set_loot_score(player.get_inventory_mut().get_loot_value());
                }
                _ => {}
            }
        }
    }

    pub(crate) async fn start_game(&mut self) -> Result<Option<GameOverChoice>, Error>{
        let mut generated = false;
        while !generated {
            let init_result = self.initialise().await;
            match init_result {
                Ok(()) => {
                    info!("Engine initialised...");
                    generated = true;
                    self.game_running = true;
                },
                Err(e) => {
                    // Rebuild the engine to reset the seed and try again
                    log::error!("Initialisation failed with error {}. Trying another map seed...", e);
                    let mut error_dialog = DialogView::new(&mut self.ui_wrapper.ui, &mut self.ui_wrapper.terminal_manager, String::from("Initialisation failed, trying another map seed..."));
                    error_dialog.begin()?;
                    self.rebuild();
                }
            }
        }

        while self.game_running {
            self.add_or_update_additional_widgets();
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

            let result = self.game_loop().await?;
            if result.is_some() {
                return Ok(result);
            }
        }
        //self.terminal_manager.terminal.clear()?;
        Ok(None)
    }

    fn build_testing_inventory(&mut self) {
        let player = self.levels.get_level_mut().characters.get_player_mut().unwrap();
        player.set_inventory(build_dev_player_inventory());
    }

    async fn attempt_player_movement(&mut self, side: Side) -> PlayerMovementResult {
        let levels = &mut self.levels;
        let level = levels.get_level_mut();
        let updated_position = level.find_player_side_position(side).clone();
        match updated_position {
            Some(pos) => {
                let level_change;
                if let Some(m) = &level.map {
                    if m.is_traversable(pos) {
                        let player = level.characters.get_player_mut().unwrap();
                        player.set_position(pos);
                    }

                    if let Some(room) = m.rooms.iter()
                        .find(|r| r.get_inside_area().contains_position(pos)) {
                        // Future TODO move to a specific controller instead?
                        level_change = self.ui_wrapper.check_room_entry_exits(room, pos);
                        let must_generate_map = levels.must_build_level(level_change.clone());
                        return PlayerMovementResult { must_generate_map, level_change: Some(level_change) };
                    }
                }

            }
            _ => {}
        }
        return PlayerMovementResult { must_generate_map: false, level_change: None };
    }

    fn handle_game_over(&mut self) -> Result<Option<GameOverChoice>, Error> {
        let player_score = self.levels.get_level_mut().characters.get_player_mut().unwrap().get_inventory_mut().get_loot_value();

        let mut menu = build_game_over_menu(
            format!("You left the dungeon.\nLoot Total: {}", player_score),
            &mut self.ui_wrapper.ui,
            &mut self.ui_wrapper.terminal_manager);
        let result = menu.begin()?;
        if let Some(game_over_choice) = result.view_specific_result {
            return Ok(Some(game_over_choice));
        }
        return Ok(None)
    }

    pub(crate) async fn handle_player_movement(&mut self, side: Side) -> Result<Option<GameOverChoice>, Error> {
        let movement_result : PlayerMovementResult = self.attempt_player_movement(side).await;

        // If the player move results in an up/down level movement, handle this
        let level_change_option = movement_result.level_change.clone();
        if let Some(level_change) = level_change_option {
            let mut map : Option<Map> = None;
            if movement_result.must_generate_map {
                map = Some(self.generate_map().await.unwrap());
            }
            let levels = &mut self.levels;

            let change_level = levels.change_level(level_change.clone(), map);
            match change_level {
                Ok(result) => {
                    match result {
                        LevelChangeResult::LevelChanged => {
                            self.respawn_player(level_change);
                        },
                        LevelChangeResult::OutOfDungeon => {
                           return self.handle_game_over();
                        }
                    }
                }
                _ => {}
            }
        }
        return Ok(None)
    }

    pub async fn menu_command(&mut self) -> Result<Option<GameOverChoice>, Error> {
        self.ui_wrapper.clear_screen()?;
        self.ui_wrapper.ui.hide_console();

        if let Some(goc) = start_menu(self, None).await.await? {
            self.ui_wrapper.ui.show_console();
            self.ui_wrapper.clear_screen()?;
            return Ok(Some(goc));
        }

        self.ui_wrapper.ui.show_console();
        self.ui_wrapper.clear_screen()?;
        Ok(None)
    }

    pub(crate) fn begin_combat(&mut self) -> Result<Option<GameOverChoice>, Error>  {
        let level = self.levels.get_level_mut();

        let characters = &level.characters;
        let player = characters.get_player().unwrap().clone();
        let mut npcs = Vec::new();
        npcs.push(characters.get_npcs().first().unwrap().clone());
        let battle_characters = Characters::new(Some(player), npcs);
        let battle = Battle { characters: battle_characters , in_progress: true };

        let view_battle = battle.clone();
        let mut combat = Combat { battle };

        let mut combat_view = CombatView::new(&mut self.ui_wrapper.ui, &mut self.ui_wrapper.terminal_manager, self.levels.get_level_mut().clone(), view_battle);
        combat_view.set_callback(Box::new(|data| {
            combat.handle_callback(data)
        }));
        combat_view.begin()?;
        Ok(None)
    }

    async fn player_turn(&mut self)  -> Result<Option<GameOverChoice>, Error> {
        let key = get_input_key()?;
        //self.terminal_manager.terminal.clear()?;
        return Ok(handle_input(self, key).await?);
    }

    fn npc_turns(&mut self)  -> Result<(), Error> {
        // TODO NPC movement
        return Ok(());
    }

    async fn game_loop(&mut self) -> Result<Option<GameOverChoice>, Error> {
        let game_over_choice = self.player_turn().await?;
        self.npc_turns()?;
        return Ok(game_over_choice);
    }
}

pub fn build_game_engine<'a, B: Backend>(terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, Error> {
    let ui = build_ui();
    let settings = build_settings();
    let rng_seed = settings.get_rng_seed().ok_or(Error::new(ErrorKind::NotFound, "Failed to retrieve the RNG seed value!"))?;
    let seed_copy = rng_seed.clone();
    let rng = Seeder::from(rng_seed).make_rng();
    Ok(GameEngine { levels: init_level_manager(seed_copy, rng), settings, ui_wrapper : UIWrapper { ui, terminal_manager }, sound_sinks: None, game_running: false })
}

pub fn build_test_game_engine<'a, B: Backend>(levels: Levels, terminal_manager : TerminalManager<B>) -> Result<GameEngine<B>, Error> {
    let ui = build_ui();
    let settings = build_settings();
    Ok(GameEngine { levels, settings, ui_wrapper : UIWrapper { ui, terminal_manager }, sound_sinks: None, game_running: false })
}

struct PlayerMovementResult {
    must_generate_map: bool,
    level_change: Option<LevelChange>
}


