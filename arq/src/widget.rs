use tui::widgets::StatefulWidget;
use tui::layout::Rect;
use tui::buffer::Buffer;
use tui::style::{Color, Style, Modifier};

pub enum WidgetType {
    Text(TextInputState),
    Dropdown(DropdownInputState),
}

pub struct Widget {
    pub state_type: WidgetType
}

#[derive(Clone)]
pub struct TextInputState {
    pub selected: bool,
    pub length: i8,
    pub input : String,
    pub name: String,
    pub input_padding: i8,
    pub selected_index: i8,
}

#[derive(Clone)]
pub struct DropdownInputState {
    pub selected: bool,
    pub name: String,
    pub chosen_option : String,
    pub options : Vec<String>
}

fn build_buffer(length: i8, input: String) -> String {
    let mut buffer = String::from("");
    for i in 0..length {
        let idx = i as usize;
        let input_char =  input.chars().nth(idx);
        match (input_char) {
            Some(s) => {
                buffer.push(s);
            }, None => {
                buffer.push(' ');
            }
        }
    }
    return buffer;
}

pub trait Focusable {
    fn focus(&mut self);
    fn unfocus(&mut self);
}

impl Focusable for WidgetType {
    fn focus(&mut self) {
        match self {
            WidgetType::Text(state) => {
                state.selected = true;
            },
            WidgetType::Dropdown(state) => {
                state.selected = true;
            }
        }
    }

    fn unfocus(&mut self) {
        match self {
            WidgetType::Text(state) => {
                state.selected = false;
            }
            WidgetType::Dropdown(state) => {
                state.selected = false;
            }
        }
    }
}

impl TextInputState {
    pub fn buffer_full(&self) -> bool {
        return self.input.len() >= self.length as usize;
    }

    pub fn add_char(&mut self, c : char) {
        if !self.buffer_full() {
            self.input.push_str(&String::from(c));
        }
    }

    pub fn delete_char(&mut self) {
        if !self.buffer_full() {
            self.input.pop();
        }
    }
}

impl StatefulWidget for TextInputState {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {

        let input_start_index = area.left() + self.name.len() as u16 + self.input_padding as u16;
        let input = self.input;
        let current_cursor_index = input_start_index + input.len() as u16;
        let max_index = input_start_index + self.length as u16;

        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default().fg(Color::White).bg(Color::Black));
        let input_buffer = build_buffer(self.length.clone(), input.clone());
        buf.set_string(input_start_index, area.top(), input_buffer, Style::default().fg(Color::Black).bg(Color::White));
        if self.selected && current_cursor_index < max_index {
            let selected_cell = buf.get_mut(current_cursor_index as u16, area.top());
            selected_cell.set_style(Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
        } else if current_cursor_index == max_index {
            let selected_input_row =  Rect::new(input_start_index, area.top(), self.length.clone() as u16, 1);
            buf.set_style(selected_input_row, Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
        }
    }
}

impl StatefulWidget for DropdownInputState {
    type State = DropdownInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default().fg(Color::White).bg(Color::Black));

        let mut index: u16 = 0;
        if self.selected {
            let selected_option = self.chosen_option;

            for opt in self.options {
                let input_offset = area.left() + self.name.clone().len() as u16 + 2;
                if opt == selected_option {
                    let selected_input_row =  Rect::new(input_offset.clone(), area.top(), 12, 1);
                    buf.set_style(selected_input_row, Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
                }
                buf.set_string(input_offset, area.top() + index.clone(), opt.clone(), Style::default().fg(Color::White).bg(Color::Black));
                index += 1;
            }
        }


    }
}