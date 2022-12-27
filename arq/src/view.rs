use std::io;
use std::io::Error;

use termion::event::Key;
use termion::input::TermRead;
use tui::Frame;
use tui::layout::Rect;

use crate::map::position::{Area, build_rectangular_area, Position};
use crate::ui::ui::get_input_key;

pub mod framehandler;
pub mod util;
pub mod callback;
pub mod character_info;
pub mod map;
pub mod world_container;
pub mod settings_menu;
pub mod game_over;
pub mod usage_line;

// A View begins an I/O loop (upon calling begin()) while rendering
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
