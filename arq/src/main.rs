use std::io;
use ui::{StartMenu, StartMenuChoice, SettingsMenuChoice};
use std::convert::TryInto;
use settings::{Toggleable};

mod terminal_manager;
mod ui;
mod menu;
mod settings;

fn handle_settings_menu_selection<B : tui::backend::Backend>(manager : &mut terminal_manager::TerminalManager<B> , ui : &mut ui::UI, settings: &mut settings::EnumSettings) -> Result<(), io::Error> {
    use crate::ui::menu::Selection;

    loop {
        let last_selection = ui.settings_menu.selection;
        ui.settings_menu.handle_input();
        let selection = ui.settings_menu.selection;
        log::info!("Selection is: {}", selection);
        if last_selection != selection {
            log::info!("Selection changed to: {}", selection);
            manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) })?;
        }

        if ui.settings_menu.exit {
            log::info!("Menu exited.");
            break;
        }

        if ui.settings_menu.selected {
            match ui.settings_menu.selection.try_into() {
                Ok(SettingsMenuChoice::FogOfWar) => {
                    match settings.settings.iter_mut().find(|x| x.name == "Fog of war") {
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


fn handle_start_menu_selection<B : tui::backend::Backend>(manager : &mut terminal_manager::TerminalManager<B> , ui : &mut ui::UI) -> Result<StartMenuChoice, io::Error> {
    use crate::ui::menu::Selection;

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
            return Ok(StartMenuChoice::Quit);
        }

        if ui.start_menu.selected {
            match ui.start_menu.selection.try_into() {
                Ok(x) => {
                    return Ok(x);
                },
                Err(_) => {}
            }
        }
    }
}

fn main<>() -> Result<(), io::Error> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let mut manager = terminal_manager::init().unwrap();
    manager.terminal.clear()?;

    let start_menu = ui::build_start_menu();
    let settings_menu = ui::build_settings_menu();
    let mut ui = ui::UI { start_menu, settings_menu, frame_size : None };

    let fog_of_war = settings::Setting { name : "Fog of war".to_string(), value : false };
    let mut enum_settings = settings::EnumSettings { settings: vec![fog_of_war] };

    loop {
        manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })?;
        let start_choice = handle_start_menu_selection(&mut manager, &mut ui)?;
        match start_choice {
            StartMenuChoice::Play => {
                log::info!("Starting game..");
            },
            StartMenuChoice::Settings => {
                log::info!("Showing settings..");
                manager.terminal.draw(|frame| { ui.draw_settings_menu(frame) })?;
                handle_settings_menu_selection(&mut manager, &mut ui, &mut enum_settings)?;
            },
            StartMenuChoice::Info => {
                log::info!("Showing info..");
            },
            StartMenuChoice::Quit => {
                break;
            }
        }
    }
    Ok(())
}