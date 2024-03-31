extern crate core;

use std::io;

use futures::executor::block_on;
use termion::input::TermRead;
use termion::raw::RawTerminal;
use tui::backend::CrosstermBackend;

use crate::engine::engine_helpers::menu::start_menu;
use crate::engine::game_engine::{build_game_engine, GameEngine};
use crate::ui::ui::StartMenuChoice::Play;
use crate::view::game_over_view::GameOverChoice;

mod global_flags;
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
    let game_engine: Result<GameEngine<CrosstermBackend<RawTerminal<std::io::Stdout>>>, std::io::Error>;
    let terminal_manager = terminal::terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    let mut engine = game_engine.unwrap();

    log::info!("Displaying start menu..");
    let mut choice = None;
    let mut game_over = false;
    while !game_over {
        engine.init()?;

        let result = start_menu(&mut engine, choice.clone()).await.await;
        match result {
            Ok(Some(goc)) => {
                match goc {
                    GameOverChoice::RESTART => {
                    engine.rebuild();
                    choice = Some(Play);
                    },
                    GameOverChoice::EXIT => {
                    game_over = true;
                    }
                }
            },
            Err(e) => {
                println!("Fatal error: {}", e);
                io::stdin().keys().next().unwrap()?;
                return Ok(())
            },
            Ok(None) => {}
        }
    }
    Ok(())
}

#[tokio::main(worker_threads = 2)]
async fn main<>() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    block_on(begin()).expect("Failure in main thread!");
}
