use tui::buffer::Cell;
use std::io::Error;
use termion::event::Key;
use tui::layout::Rect;
use tui::Frame;
use tui::widgets::{Block, Borders};

use crate::map::Map;
use crate::ui::{UI, FrameHandler, FrameData};
use crate::terminal::terminal_manager::TerminalManager;
use crate::terminal::colour_mapper;
use crate::character::Character;
use crate::view::{View, InputHandler, InputResult, GenericInputResult};
use crate::map::position::Area;
use crate::view::container_view::ContainerViewInputResult;


pub struct ConsoleView {
}

pub struct ConsoleBuffer {

}

impl ConsoleView {
}

impl <B : tui::backend::Backend> FrameHandler<B, ConsoleBuffer> for ConsoleView {
    fn handle_frame(&mut self, frame: &mut Frame<B>, mut data: FrameData<ConsoleBuffer>) {
        let frame_size = data.get_frame_size().clone();
        let window_block = Block::default()
            .borders(Borders::ALL);
        frame.render_widget(window_block, frame_size);
    }
}

impl InputHandler<String> for ConsoleView {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<String>, Error> {
        let continue_result = InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        };
        return Ok(continue_result);
    }
}