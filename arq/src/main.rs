use std::io;
use ui::{StartMenu, StartMenuChoice};
use std::convert::TryInto;
mod terminal_manager;
mod ui;
mod menu;

fn main<>() -> Result<(), io::Error> {
    use crate::ui::menu::Selection;

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let mut manager = terminal_manager::init().unwrap();
    manager.terminal.clear()?;

    let start_menu = ui::build_start_menu();
    let mut ui = ui::UI { start_menu, frame_size : None };

    manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })?;
    loop {
        let last_selection = ui.start_menu.selection;
        ui.start_menu.handle_input();
        let selection = ui.start_menu.selection;
        log::info!("Selection is: {}", selection);
        if last_selection != selection {
            log::info!("Selection changed to: {}", selection);
            manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })?;
        }

        if ui.start_menu.exit {
            log::info!("Menu exited.");
            break;
        }

        if ui.start_menu.selected {
            match ui.start_menu.selection.try_into() {
                Ok(StartMenuChoice::Play) => {
                    log::info!("Starting game..");
                },
                Ok(StartMenuChoice::Settings) => {
                    log::info!("Showing settings..");
                },
                Ok(StartMenuChoice::Info) => {
                    log::info!("Showing info..");
                },
                Ok(StartMenuChoice::Quit) => {
                    break;
                },
                Err(_) => {}
            }
        }
    }
    return Ok(());
}