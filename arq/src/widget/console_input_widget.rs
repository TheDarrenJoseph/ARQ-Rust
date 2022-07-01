use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::widgets::StatefulWidget;

use crate::widget::{build_buffer, Widget, WidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct ConsoleInputState {
    pub selected: bool,
    length: i8,
    input : String,
    input_padding: i8,
    selected_index: i8,
}

pub fn build_console_input(length: i8, input: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Console( ConsoleInputState { selected: false, length, input, input_padding,  selected_index: 0 });
    Widget{ state_type: name_input_state}
}

impl ConsoleInputState {
    pub fn buffer_full(&self) -> bool {
        return self.input.len() >= self.length.clone() as usize;
    }

    pub fn add_char(&mut self, c : char) {
        if !self.buffer_full() {
            self.input.push_str(&String::from(c));
        }
    }

    pub fn delete_char(&mut self) {
        if self.input.len() > 0 {
            self.input.pop();
        }
    }

    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    pub fn set_input(&mut self, input: String)  {
        self.input = input
    }

}

impl StatefulWidget for ConsoleInputState {
    type State = ConsoleInputState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let input_start_index = area.left() + self.input_padding as u16;
        let input = self.input;
        let current_cursor_index = input_start_index + input.len() as u16;
        let max_index = input_start_index + self.length as u16;
        let input_buffer = build_buffer(self.length.clone(), input.clone());
        let mut line_no = 0;
        for line in input_buffer.lines() {
            if line_no < area.height {
                buf.set_string(input_start_index, area.top() + line_no, line, Style::default());
                line_no += 1;
            } else {
                break;
            }
        }

        if self.selected && current_cursor_index < max_index {
            let selected_cell = buf.get_mut(current_cursor_index as u16, area.top());
            selected_cell.set_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }
    }
}
