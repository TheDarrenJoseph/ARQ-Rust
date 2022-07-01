use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::widgets::StatefulWidget;

use crate::widget::{build_buffer, Widget, WidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct NumberInputState {
    pub selected: bool,
    pub editable: bool,
    length: i8,
    input : i32,
    min: i32,
    max: i32,
    pub name: String,
    input_padding: i8
}

pub fn build_number_input(editable: bool, length: i8, name: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Number( NumberInputState { selected: false, editable, length, input: 0, min: 0, max: 100, name, input_padding});
    Widget{ state_type: name_input_state}
}

pub fn build_number_input_with_value(editable: bool, input: i32, length: i8, name: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Number( NumberInputState { selected: false, editable, length, input, min: 0, max: 100, name, input_padding});
    Widget{ state_type: name_input_state}
}

impl NumberInputState {
    pub fn increment(&mut self) {
        if self.input < self.max {
            self.input += 1;
        }
    }

    pub fn decrement(&mut self) {
        if self.input > self.min {
            self.input -= 1;
        }
    }
    pub fn get_input(&mut self) -> i32 {
        self.input.clone()
    }

    pub fn set_input(&mut self, input: i32) {
        if input >= self.min && self.input <= self.max {
            self.input = input
        }
    }

    pub fn get_max(&mut self) -> i32 {
        self.max.clone()
    }

    pub fn set_max(&mut self, max: i32) {
        if max > self.min {
            self.max = max;
        }
    }
}

impl StatefulWidget for NumberInputState {
    type State = NumberInputState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default());
        let input_start_index = area.left() + self.name.len() as u16 + self.input_padding as u16;
        let input_text = if self.editable { "<- ".to_string() + &self.input.clone().to_string() + &" ->".to_string() } else { self.input.clone().to_string() };
        let input_buffer = build_buffer(input_text.len() as i8, input_text);
        let style = if self.selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
        buf.set_string(input_start_index, area.top(), input_buffer, style);

    }
}
