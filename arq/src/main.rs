use std::io;
use ui::{StartMenu, StartMenuChoice, SettingsMenuChoice};
use std::convert::TryInto;
use settings::{Toggleable};
use termion::input::TermRead;

mod terminal_manager;
mod ui;
mod items;
mod container;
mod menu;
mod settings;
mod tile;
mod map;
mod map_view;

use crate::container::{ContainerType};
use crate::menu::Selection;
use crate::terminal_manager::TerminalManager;
use crate::map_view::MapView;

use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::buffer::Cell;
use termion::raw::RawTerminal;
use std::time::Duration;

struct GameEngine  {
    terminal_manager : TerminalManager<TermionBackend<RawTerminal<io::Stdout>>>,
    ui : ui::UI,
    settings : settings::EnumSettings
}

impl GameEngine {

    fn draw_settings_menu(&mut self) {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) });
    }

    fn draw_start_menu(&mut self) {
        let mut ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_start_menu(frame) });
    }

    fn handle_settings_menu_selection(&mut self) -> Result<(), io::Error> {
        loop {
            let mut ui = &self.ui;
            let last_selection = self.ui.settings_menu.selection;
            let key = io::stdin().keys().next().unwrap().unwrap();
            self.ui.settings_menu.handle_input(key);
            let selection = self.ui.settings_menu.selection;
            log::info!("Selection is: {}", selection);
            if last_selection != selection {
                log::info!("Selection changed to: {}", selection);
                self.draw_settings_menu();
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
            self.draw_start_menu();
            let start_choice = self.handle_start_menu_selection()?;
            match start_choice {
                StartMenuChoice::Play => {
                    log::info!("Starting game..");
                    self.start_game();
                    break;
                },
                StartMenuChoice::Settings => {
                    log::info!("Showing settings..");
                    let ui = &mut self.ui;
                    self.terminal_manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) })?;
                    self.handle_settings_menu_selection()?;
                    break;
                },
                StartMenuChoice::Info => {
                    log::info!("Showing info..");
                    break;
                },
                StartMenuChoice::Quit => {
                    break;
                }
            }
        }
        Ok(())
    }

    fn start_game(&mut self) {
        let tile_library = crate::tile::build_library();

        let room = &tile_library[2];
        let wall = &tile_library[3];

        let mut map = crate::map::Map { tiles : vec![
            vec![ wall,  wall,  wall],
            vec![ wall,  room,  wall],
            vec![ wall,  wall, wall]
            ]
        };

        let mut map_view = MapView{ map, terminal_manager : &mut self.terminal_manager };
        map_view.draw_map();
    }
}

fn build_game_engine(mut terminal_manager : TerminalManager<TermionBackend<RawTerminal<std::io::Stdout>>>) -> Result<GameEngine, io::Error> {
    let start_menu = menu::build_start_menu();
    let settings_menu = menu::build_settings_menu();
    let mut ui = ui::UI { start_menu, settings_menu, frame_size : None };

    terminal_manager.terminal.clear()?;

    let fog_of_war = settings::Setting { name : "Fog of war".to_string(), value : false };
    let mut settings = settings::EnumSettings { settings: vec![fog_of_war] };

    Ok(GameEngine { terminal_manager, ui, settings })
}

fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let mut game_engine : Result<GameEngine, std::io::Error>;
    let mut terminal_manager = terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    //let _container = container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    game_engine.unwrap().start_menu()
}