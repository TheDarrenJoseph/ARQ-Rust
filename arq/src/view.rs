use std::io::Error;
use std::io;

pub mod character_view;
pub mod container_view;
pub mod character_info_view;
pub mod map_view;
use termion::event::Key;
use termion::input::TermRead;

use crate::map::position::{Area, Position, build_rectangular_area};
use tui::Frame;
use tui::layout::Rect;

pub trait View {
    fn begin(&mut self) -> Result<bool, Error>;
    fn draw(&mut self, area : Option<Rect>) -> Result<(), Error>;
    fn handle_input(&mut self, input : Option<Key>) -> Result<bool, Error>;
}

pub struct InputResult {
    pub(crate) done: bool,
    pub(crate) requires_view_refresh: bool
}

pub trait InputHandler {
    fn handle_input(&mut self, input : Option<Key>) -> Result<InputResult, Error>;
}

fn map_rect_to_area(rect: Rect) -> Area {
    let start_position = Position { x: rect.x.clone(), y : rect.y.clone()};
    build_rectangular_area(start_position, rect.width.clone(), rect.height.clone())
}

pub fn resolve_area<B : tui::backend::Backend>(area: Option<Rect>, frame: &Frame<B>) -> Rect {
    let mut frame_area;
    match area {
        Some(a) => { frame_area = a },
        _ => { frame_area = frame.size() }
    }
    frame_area
}

pub fn resolve_input(input : Option<Key>) -> Key {
    match input {
        Some(input_key) => {
            input_key
        },
        None => {
            io::stdin().keys().next().unwrap().unwrap()
        }
    }

}