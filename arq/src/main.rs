use std::io;

mod terminal_manager;
mod ui;

fn main<>() -> Result<(), io::Error> {
    let mut terminal_manager = terminal_manager::init().unwrap();
    return terminal_manager.terminal.draw(|frame| { ui::draw_start_menu(frame) });
}