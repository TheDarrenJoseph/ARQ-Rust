extern crate core;

use std::io;
use std::io::SeekFrom::Start;

use termion::raw::RawTerminal;
use tui::backend::TermionBackend;

use crate::engine::game_engine::{build_game_engine, GameEngine};
use crate::ui::ui::StartMenuChoice::Play;
use crate::view::game_over::GameOverChoice;
use crate::view::game_over::GameOverChoice::EXIT;

mod error_utils;
mod engine;
mod terminal;
mod ui;
mod menu;
mod settings;
mod view;
mod character;
mod characters;
mod widget;
mod test;
mod list_selection;
mod util;
mod sound;
pub mod map;


fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let mut game_engine: Result<GameEngine<TermionBackend<RawTerminal<std::io::Stdout>>>, std::io::Error>;
    let mut terminal_manager = terminal::terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    let mut engine = game_engine.unwrap();
    log::info!("Displaying start menu..");

    let mut choice = None;
    let mut game_over = false;
    while !game_over {
        if let Some(goc) = engine.start_menu(choice.clone())? {
            engine.ui_wrapper.clear_screen();
            match goc {
                GameOverChoice::RESTART => {
                    engine.rebuild();
                    choice = Some(Play);
                },
                GameOverChoice::EXIT => {
                    game_over = true;
                }
            }
        }
    }
    Ok(())
}
