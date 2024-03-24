use std::error::Error as StdError;
use std::io;
use std::io::Error;
use futures::future::err;

use termion::event::Key;
use termion::input::TermRead;

use tui::Frame;
use tui::layout::Rect;
use crate::build_paragraph;

use crate::map::position::{Area, build_rectangular_area, Position};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{get_input_key, UI};
use crate::ui::ui_util::{build_paragraph_multi, check_display_size, MIN_AREA};

pub mod framehandler;
pub mod util;
pub mod character_info_view;
pub mod map_view;
pub mod world_container_view;
pub mod settings_menu_view;
pub mod game_over_view;
pub mod combat_view;
pub mod model;
pub mod dialog_view;
pub mod menu_view;

/*
    A "View" is:
     * Responsible for managing a particular screen of the UI
     * Something that usually contains a series of FrameHandler(s) that own widgets / state that is then rendered to each UI frame (the entire window per render)
     * Something that often behaves as a proxy between the engine and underlying framehandlers
     * Able to take control of the UI and I/O (rendering, keyboard input, etc) until it exits
        * In this manner a View can take control / begin an I/O loop (upon calling begin()) while rendering itself via framhandlers

    From a data perspective, a View has direct access to both:
     * UI (ARQ's base UI, window areas, and it's elements (widgets))
     * The terminal manager (from tui-rs) which is what allows us access to each frame via draw() which is then passed to each framehandler, this also allows direct access to the terminal display (character printing etc)
 */
pub trait View<T>  {
    fn begin(&mut self) -> Result<InputResult<T>, Error>;
    fn draw(&mut self, area : Option<Area>) -> Result<(), Error>;
}

pub trait InputHandler<T> {
    fn handle_input(&mut self, input : Option<Key>) -> Result<InputResult<T>, Error>;
}

pub struct GenericInputResult {
    pub(crate) done: bool,
    pub(crate) requires_view_refresh: bool
}

pub struct InputResult<T> {
    pub(crate) generic_input_result : GenericInputResult,
    pub(crate) view_specific_result : Option<T>
}

fn map_rect_to_area(rect: Rect) -> Area {
    let start_position = Position { x: rect.x.clone(), y : rect.y.clone()};
    build_rectangular_area(start_position, rect.width.clone(), rect.height.clone())
}

pub fn resolve_area<B : tui::backend::Backend>(area: Option<Rect>, frame: &Frame<B>) -> Rect {
    return match area {
        Some(a) => { a },
        _ => { frame.size() }
    }
}

pub fn resolve_input(input : Option<Key>) -> Result<Key, io::Error> {
    match input {
        Some(input_key) => {
            Ok(input_key)
        },
        _ => {
            Ok(get_input_key()?)
        }
    }

}

pub fn verify_display_size<B : tui::backend::Backend>(terminal_manager : &mut TerminalManager<B>) {
    loop {
        let frame_size = terminal_manager.terminal.size().unwrap();
        let result = check_display_size(Some(Area::from_rect(frame_size)));
        match result {
            Err(e) => {
                terminal_manager.terminal.clear();
                let error_paragraph = build_paragraph_multi(
                    vec![String::from(
                        "Window too small."),
                         format!("Minimum is {}x{}", MIN_AREA.width, MIN_AREA.height),
                         String::from("Please resize and hit any key to check again.") ]);
                terminal_manager.terminal.draw(|frame|{
                    frame.render_widget(error_paragraph, frame_size);
                });
                io::stdin().keys().next().unwrap().unwrap();
            },
            _ => {
                return;
            }
        }
    }
}

