use std::io;
use ui::{StartMenu};

mod terminal_manager;
mod ui;
mod menu;

fn main<>() -> Result<(), io::Error> {
    use crate::ui::menu::Selection;

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let mut manager = terminal_manager::init().unwrap();
    manager.terminal.clear()?;

    let start_menu = ui::build_start_menu();
    let mut ui = ui::UI { menu : start_menu, frame_size : None };

    manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })?;
    loop {
        let last_selection = ui.menu.selection;
        ui.menu.handle_input();
        let selection = ui.menu.selection;
        log::info!("Selection is: {}", selection);
        if last_selection != selection {
            log::info!("Selection changed to: {}", selection);
            manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })?;
        }

        if ui.menu.exit {
            log::info!("Menu exited.");
            break;
        }

        if ui.menu.selected {
            log::info!("Menu item {} selected.", selection);
            break;
        }
    }


    return Ok(());
}