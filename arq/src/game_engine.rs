use termion::input::TermRead;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use termion::event::Key;
use std::io;
use std::convert::TryInto;

use crate::ui;
use crate::ui::Draw;
use crate::settings;
use crate::settings::Toggleable;
use crate::menu;
use crate::menu::{Selection};
use crate::ui::{SettingsMenuChoice, StartMenuChoice};
use crate::map_view::MapView;
use crate::character_view::{CharacterView, CharacterViewFrameHandler, ViewMode};
use crate::map_generator::build_generator;
use crate::terminal_manager::TerminalManager;
use crate::position::{Position, build_rectangular_area};
use crate::character::{Character, build_player};

pub struct GameEngine  {
    terminal_manager : TerminalManager<TermionBackend<RawTerminal<io::Stdout>>>,
    ui : ui::UI,
    settings : settings::EnumSettings,
    game_running : bool,
    characters : Vec<Character>,
}

impl GameEngine {

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
            self.draw_start_menu()?;
            let start_choice = self.handle_start_menu_selection()?;
            match start_choice {
                StartMenuChoice::Play => {
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
        self.terminal_manager.terminal.clear()?;
        Ok(())
    }

    fn build_characters(&self) -> Vec<Character> {
        let position = Position { x: 1, y: 1};
        let player = build_player("Player".to_string(), position);

        let mut characters = Vec::new();
        characters.push(player);
        return characters;
    }

    fn start_game(&mut self) -> Result<(), io::Error>{
        let map_area = build_rectangular_area(Position { x: 0, y: 0 }, 40, 20);
        let mut map_generator = build_generator(map_area);
        let map = &map_generator.generate();
        let mut characters = self.build_characters();

        let mut character_created = false;
        self.game_running = true;
        while self.game_running {
            if !character_created {
                let frame_handler = CharacterViewFrameHandler { widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::CREATION};
                let mut character_view = CharacterView { character: characters.get(0).unwrap().clone(), terminal_manager: &mut self.terminal_manager, frame_handler};
                character_view.draw()?;

                while !character_created {
                    character_created = character_view.handle_input().unwrap();
                    character_view.draw();
                }

                let updated_character = character_view.get_character();
                characters[0] = updated_character;
                self.characters = characters.clone();
            }

            let mut map_view = MapView { map, characters: characters.clone(), terminal_manager: &mut self.terminal_manager };
            map_view.draw_map()?;
            map_view.draw_characters()?;

            self.game_loop()?;
        }
        Ok(())
    }

    fn game_loop(&mut self) -> Result<(), io::Error> {
        let key = io::stdin().keys().next().unwrap().unwrap();
        match key {
            Key::Char('q') => {
                self.terminal_manager.terminal.clear()?;
                self.start_menu()?;
                self.terminal_manager.terminal.clear()?;
            },
            Key::Char('a') => {
                self.terminal_manager.terminal.clear()?;
                let frame_handler = CharacterViewFrameHandler { widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::VIEW };
                let mut character_view = CharacterView { character: self.characters.get(0).unwrap().clone(), terminal_manager: &mut self.terminal_manager, frame_handler};
                character_view.draw()?;
                let key = io::stdin().keys().next().unwrap().unwrap();
            }
            _ => {}
        }

        Ok(())
    }
}

pub fn build_game_engine(mut terminal_manager : TerminalManager<TermionBackend<RawTerminal<std::io::Stdout>>>) -> Result<GameEngine, io::Error> {
    let start_menu = menu::build_start_menu();
    let settings_menu = menu::build_settings_menu();
    let ui = ui::UI { start_menu, settings_menu, frame_size : None };

    terminal_manager.terminal.clear()?;

    let fog_of_war = settings::Setting { name : "Fog of war".to_string(), value : false };
    let settings = settings::EnumSettings { settings: vec![fog_of_war] };

    Ok(GameEngine { terminal_manager, ui, settings, game_running: false, characters: Vec::new()})
}
