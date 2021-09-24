use termion::input::TermRead;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;

use settings::Toggleable;
use std::convert::TryInto;
use std::io;
use ui::{SettingsMenuChoice, StartMenu, StartMenuChoice};

use crate::map_view::MapView;
use crate::map_generator::build_generator;
use crate::menu::Selection;
use crate::terminal_manager::TerminalManager;
use crate::position::{Position, build_square_area, build_rectangular_area};

mod terminal_manager;
mod ui;
mod items;
mod container;
mod menu;
mod settings;
mod tile;
mod map;
mod map_view;
mod map_generator;
mod position;
mod door;
mod room;
mod colour_mapper;

struct GameEngine  {
    terminal_manager : TerminalManager<TermionBackend<RawTerminal<io::Stdout>>>,
    ui : ui::UI,
    settings : settings::EnumSettings
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
                    log::info!("Starting game..");
                    self.start_game()?;
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

    fn start_game(&mut self) -> Result<(), io::Error>{
        let map_area = build_rectangular_area(Position {x: 0, y: 0}, 40, 20);
        let mut map_generator = build_generator(map_area);

        let map = map_generator.generate();

        let mut map_view = MapView{ map, terminal_manager : &mut self.terminal_manager };

        map_view.draw_map()?;
        Ok(())
    }
}

fn build_game_engine(mut terminal_manager : TerminalManager<TermionBackend<RawTerminal<std::io::Stdout>>>) -> Result<GameEngine, io::Error> {
    let start_menu = menu::build_start_menu();
    let settings_menu = menu::build_settings_menu();
    let ui = ui::UI { start_menu, settings_menu, frame_size : None };

    terminal_manager.terminal.clear()?;

    let fog_of_war = settings::Setting { name : "Fog of war".to_string(), value : false };
    let settings = settings::EnumSettings { settings: vec![fog_of_war] };

    Ok(GameEngine { terminal_manager, ui, settings })
}

fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let game_engine : Result<GameEngine, std::io::Error>;
    let terminal_manager = terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    //let _container = container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    game_engine.unwrap().start_menu()
}