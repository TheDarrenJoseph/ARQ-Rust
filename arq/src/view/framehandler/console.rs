use tui::buffer::Cell;
use std::io::Error;
use termion::event::Key;
use tui::layout::Rect;
use tui::Frame;
use tui::widgets::{Block, Borders};
use std::convert::TryInto;

use crate::map::Map;
use crate::ui::{UI, FrameHandler, FrameData};
use crate::terminal::terminal_manager::TerminalManager;
use crate::terminal::colour_mapper;
use crate::character::Character;
use crate::view::{View, InputHandler, InputResult, GenericInputResult};
use crate::map::position::Area;
use crate::view::framehandler::container::ContainerFrameHandlerInputResult;
use crate::widget::console_input_widget::{build_console_input, ConsoleInputState};
use crate::widget::WidgetType;


pub struct ConsoleFrameHandler {
    pub buffer: ConsoleBuffer
}

pub struct ConsoleBuffer {
    pub content : String
}

impl ConsoleFrameHandler {
}

impl <B : tui::backend::Backend> FrameHandler<B, ConsoleBuffer> for ConsoleFrameHandler {
    fn handle_frame(&mut self, frame: &mut Frame<B>, mut data: FrameData<ConsoleBuffer>) {
        let frame_size : Rect = data.get_frame_size().clone();
        let window_block = Block::default()
            .borders(Borders::ALL);
        frame.render_widget(window_block, frame_size);

        let adjusted_text_width = frame_size.width - 2;
        let length : i8 = if adjusted_text_width >= i8::MAX as u16 { i8::MAX } else { adjusted_text_width.try_into().unwrap() };
        let mut console_input = build_console_input(length, String::from(""), self.buffer.content.clone(), 0);
        let text_area = Rect::new(frame_size.x +  1, frame_size.y + 1, frame_size.width - 2 , frame_size.height - 2 );
        match &mut console_input.state_type {
            WidgetType::Console(w) => {
                frame.render_stateful_widget(w.clone(), text_area, &mut w.clone());
            },
            _ => {}
        }
    }
}

impl InputHandler<String> for ConsoleFrameHandler {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<String>, Error> {
        let continue_result = InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        };
        return Ok(continue_result);
    }
}