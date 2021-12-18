use tui::widgets::StatefulWidget;
use tui::layout::Rect;
use tui::buffer::Buffer;
use tui::style::{Style, Modifier};

use crate::widget::{Widget, WidgetType, build_buffer};

#[derive(Clone)]
#[derive(Debug)]
pub struct ConsoleInputState {
    pub selected: bool,
    length: i8,
    input : String,
    name: String,
    input_padding: i8,
    selected_index: i8,
}

pub fn build_console_input(length: i8, name: String, input: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Console( ConsoleInputState { selected: false, length, input, name, input_padding,  selected_index: 0 });
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

    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }

}

impl StatefulWidget for ConsoleInputState {
    type State = ConsoleInputState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let input_start_index = area.left() + self.name.len() as u16 + self.input_padding as u16;
        let input = self.input;
        let current_cursor_index = input_start_index + input.len() as u16;
        let max_index = input_start_index + self.length as u16;

        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default());
        let input_buffer = build_buffer(self.length.clone(), input.clone());
        buf.set_string(input_start_index, area.top(), input_buffer, Style::default());
        if self.selected && current_cursor_index < max_index {
            let selected_cell = buf.get_mut(current_cursor_index as u16, area.top());
            selected_cell.set_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }
    }
}
