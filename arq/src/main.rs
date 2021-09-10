use std::io;

mod terminal_manager;
mod ui;

fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let mut manager = terminal_manager::init().unwrap();
    manager.terminal.clear()?;
    manager.terminal.draw(|frame| { ui::draw_start_menu(frame) });
    terminal_manager::get_input_key();
    return Ok(());
}