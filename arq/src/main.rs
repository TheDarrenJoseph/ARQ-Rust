mod game_engine;
mod terminal_manager;
mod ui;
mod items;
mod container;
mod menu;
mod settings;
mod tile;
mod map;
mod view;
mod map_generator;
mod position;
mod character;
mod widget;
mod test;
mod door;
mod room;
mod colour_mapper;
mod pathfinding;
mod map_view;
mod character_view;
mod container_view;

use std::io;

use crate::game_engine::{GameEngine, build_game_engine};

fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let game_engine : Result<GameEngine, std::io::Error>;
    let terminal_manager = terminal_manager::init().unwrap();
    game_engine = build_game_engine(terminal_manager);
    //let _container = container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    game_engine.unwrap().start_menu()?;
    Ok(())
}