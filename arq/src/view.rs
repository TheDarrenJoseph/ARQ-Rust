use std::io;
use std::io::Error;

use termion::event::Key;
use termion::input::TermRead;
use tui::Frame;
use tui::layout::Rect;

use crate::map::position::{Area, build_rectangular_area, Position};



pub mod framehandler;
pub mod callback;
pub mod character_info;
pub mod map;
pub mod world_container;
pub mod settings_menu;

// A View begins an I/O loop (upon calling begin()) while rendering
pub trait View<'b, COM: 'b>  {
    fn begin(&mut self) -> Result<bool, Error>;
    fn draw(&mut self, area : Option<Area>) -> Result<(), Error>;
    fn handle_input(&mut self, input : Option<Key>) -> Result<bool, Error>;
}

pub struct GenericInputResult {
    pub(crate) done: bool,
    pub(crate) requires_view_refresh: bool
}

pub struct InputResult<T> {
    pub(crate) generic_input_result : GenericInputResult,
    pub(crate) view_specific_result : Option<T>
}

pub trait InputHandler<T> {
    fn handle_input(&mut self, input : Option<Key>) -> Result<InputResult<T>, Error>;
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

pub fn resolve_input(input : Option<Key>) -> Key {
    match input {
        Some(input_key) => {
            input_key
        },
        _ => {
            io::stdin().keys().next().unwrap().unwrap()
        }
    }

}
