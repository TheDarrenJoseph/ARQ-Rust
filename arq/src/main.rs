extern crate core;

use std::convert::TryInto;
use std::io;
use std::io::SeekFrom::Start;
use futures::executor::block_on;

use termion::raw::RawTerminal;
use tui::backend::TermionBackend;

use crate::engine::game_engine::{build_game_engine, GameEngine};
use crate::ui::ui::StartMenuChoice::Play;
use crate::view::game_over_view::GameOverChoice;
use crate::view::game_over_view::GameOverChoice::EXIT;

mod error;
mod engine;
mod character;
mod terminal;
mod ui;
mod menu;
mod settings;
mod view;
mod widget;
mod test;
mod item_list_selection;
mod option_list_selection;
mod util;
mod progress;
mod sound;
pub mod map;

async fn begin() -> Result<(), io::Error> {
    let mut game_engine: Result<GameEngine<TermionBackend<RawTerminal<std::io::Stdout>>>, std::io::Error>;
    let mut terminal_manager = terminal::terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    let mut engine = game_engine.unwrap();
    log::info!("Displaying start menu..");

    let mut choice = None;
    let mut game_over = false;
    while !game_over {
        if let Ok(Some(goc)) = engine.start_menu(choice.clone()).await.await {
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

#[tokio::main(worker_threads = 2)]
async fn main<>() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    block_on(begin());
}
