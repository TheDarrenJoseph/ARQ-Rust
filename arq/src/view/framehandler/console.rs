use std::convert::TryInto;
use std::io::Error;

use termion::event::Key;
use tui::Frame;
use tui::layout::Rect;
use tui::widgets::{Block, Borders};
use crate::error::errors::ErrorWrapper;

use crate::view::{GenericInputResult, InputHandler, InputResult};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::widget::stateful::console_input_widget::build_console_input;
use crate::widget::StatefulWidgetType;

/*
 This frame handler displays the console window to display contextual info/get input
 */
pub struct ConsoleFrameHandler {
    pub buffer: ConsoleBuffer
}

pub struct ConsoleBuffer {
    pub content : String
}

impl ConsoleFrameHandler {
}

impl <B : tui::backend::Backend> FrameHandler<B, ConsoleBuffer> for ConsoleFrameHandler {
    fn handle_frame(&mut self, frame: &mut Frame<B>, data: FrameData<ConsoleBuffer>) {
        let frame_size : Rect = data.get_frame_area().clone();
        let window_block = Block::default()
            .borders(Borders::ALL);
        frame.render_widget(window_block, frame_size);

        let adjusted_text_width = frame_size.width - 2;
        let length : i8 = if adjusted_text_width >= i8::MAX as u16 { i8::MAX } else { adjusted_text_width.try_into().unwrap() };
        let console_input = build_console_input(length, self.buffer.content.clone(), 0);
        let text_area = Rect::new(frame_size.x +  1, frame_size.y + 1, frame_size.width - 2 , frame_size.height - 2 );

        if let StatefulWidgetType::Console(w) = console_input.state_type {
            frame.render_stateful_widget(w.clone(), text_area, &mut w.clone());
        }
    }
}

impl InputHandler<String> for ConsoleFrameHandler {
    fn handle_input(&mut self, _: Option<Key>) -> Result<InputResult<String>, ErrorWrapper> {
        return Ok(InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        });
    }
}