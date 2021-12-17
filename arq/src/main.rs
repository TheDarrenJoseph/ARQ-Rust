use std::io;

use crate::engine::game_engine::{build_game_engine, GameEngine};
use tui::backend::TermionBackend;
use termion::raw::RawTerminal;

mod engine;
mod terminal;
mod ui;
mod menu;
mod settings;
mod view;
mod character;
mod widget;
mod test;
mod list_selection;
pub mod map;

fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let game_engine : Result<GameEngine<TermionBackend<RawTerminal<std::io::Stdout>>>, std::io::Error>;
    let terminal_manager = terminal::terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    //let _container = container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    log::info!("Displaying start menu..");
    game_engine.unwrap().start_menu()?;
    Ok(())
}
