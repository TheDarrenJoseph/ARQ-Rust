use tui::widgets::StatefulWidget;
use tui::layout::Rect;
use tui::buffer::Buffer;
use tui::style::{Color, Style, Modifier};

#[derive(Clone)]
pub struct TextInput {
    pub name: String,
    pub input_padding: i8,
    pub length: i8,
    pub selected: bool,
    pub selected_index: i8,
    pub state: TextInputState
}

#[derive(Clone)]
pub struct TextInputState {
    pub input : String
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

impl TextInput {
    pub fn buffer_full(&self) -> bool {
        return self.state.input.len() >= self.length as usize;
    }

    pub fn focus(&mut self) {
        self.selected = true;
    }

    pub fn unfocus(&mut self) {
        self.selected = false;
    }

    pub fn add_char(&mut self, c : char) {
        if !self.buffer_full() {
            self.state.input.push_str(&String::from(c));
        }
    }

    pub fn delete_char(&mut self) {
        if !self.buffer_full() {
            self.state.input.pop();
        }
    }
}

impl StatefulWidget for TextInput {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {

        let input_start_index = area.left() + self.name.len() as u16 + self.input_padding as u16;
        let input = self.state.input;
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