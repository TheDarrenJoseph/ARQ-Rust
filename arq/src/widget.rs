use tui::widgets::StatefulWidget;
use tui::layout::Rect;
use tui::buffer::Buffer;
use tui::style::{Style, Modifier};

fn build_buffer(length: i8, input: String) -> String {
    let mut buffer = String::from("");
    for i in 0..length {
        let idx = i as usize;
        let input_char =  input.chars().nth(idx);
        match input_char {
            Some(s) => {
                buffer.push(s);
            }, None => {
                buffer.push(' ');
            }
        }
    }
    return buffer;
}


#[derive(Debug)]
pub enum WidgetType {
    Text(TextInputState),
    Number(NumberInputState),
    Dropdown(DropdownInputState),
}

pub struct Widget {
    pub state_type: WidgetType
}

#[derive(Clone)]
#[derive(Debug)]
pub struct TextInputState {
    selected: bool,
    length: i8,
    input : String,
    name: String,
    input_padding: i8,
    selected_index: i8,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct NumberInputState {
    selected: bool,
    editable: bool,
    length: i8,
    input : i32,
    min: i32,
    max: i32,
    pub name: String,
    input_padding: i8
}

#[derive(Clone)]
#[derive(Debug)]
pub struct DropdownInputState {
    pub selected: bool,
    show_options: bool,
    name: String,
    options : Vec<String>,
    selected_index: i8,
    chosen_option : String
}

pub trait Focusable {
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn is_focused(&mut self) -> bool;
}

pub fn build_text_input(length: i8, name: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Text( TextInputState { selected: false, length, input: "".to_string(), name, input_padding,  selected_index: 0 });
    Widget{ state_type: name_input_state}
}

pub fn build_number_input(editable: bool, length: i8, name: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Number( NumberInputState { selected: false, editable, length, input: 0, min: 0, max: 100, name, input_padding});
    Widget{ state_type: name_input_state}
}

pub fn build_number_input_with_value(editable: bool, input: i32, length: i8, name: String, input_padding: i8) -> Widget {
    let name_input_state = WidgetType::Number( NumberInputState { selected: false, editable, length, input, min: 0, max: 100, name, input_padding});
    Widget{ state_type: name_input_state}
}

pub fn build_dropdown(name: String, options: Vec<String>) -> Widget {
    let state = WidgetType::Dropdown( DropdownInputState { selected: false, show_options: false, name, selected_index: 0, chosen_option: options[0].to_string(), options});
    Widget{ state_type: state}
}


impl TextInputState {
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

impl DropdownInputState {
    pub fn select_next(&mut self) {
        if self.selected_index < self.options.len() as i8 - 1 {
            self.selected_index += 1;
            self.chosen_option = self.options[self.selected_index.clone() as usize].clone();
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.chosen_option = self.options[self.selected_index.clone() as usize].clone();
        }
    }

    pub fn get_selection(&self) -> String {
        return self.chosen_option.clone();
    }

    pub fn is_showing_options(&self) -> bool {
        return self.show_options.clone();
    }

    pub fn toggle_show(&mut self) {
        self.show_options = !self.show_options.clone();
    }
}

impl Focusable for WidgetType {
    fn focus(&mut self) {
        match self {
            WidgetType::Text(state) => {
                state.selected = true;
            },
            WidgetType::Number(state) => {
                state.selected = true;
            }
            WidgetType::Dropdown(state) => {
                state.selected = true;
            }
        }
    }

    fn unfocus(&mut self) {
        match self {
            WidgetType::Text(state) => {
                state.selected = false;
            },
            WidgetType::Number(state) => {
                state.selected = false;
            }
            WidgetType::Dropdown(state) => {
                state.selected = false;
            }
        }
    }

    fn is_focused(&mut self) -> bool {
        match self {
            WidgetType::Text(state) => {
                state.selected.clone()
            },
            WidgetType::Number(state) => {
                state.selected.clone()
            }
            WidgetType::Dropdown(state) => {
                state.selected.clone()
            }
        }
    }
}

impl StatefulWidget for TextInputState {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {

        let input_start_index = area.left() + self.name.len() as u16 + self.input_padding as u16;
        let input = self.input;
        let current_cursor_index = input_start_index + input.len() as u16;
        let max_index = input_start_index + self.length as u16;

        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default());
        let input_buffer = build_buffer(self.length.clone(), input.clone());
        buf.set_string(input_start_index, area.top(), input_buffer, Style::default().add_modifier(Modifier::REVERSED));
        if self.selected && current_cursor_index < max_index {
            let selected_cell = buf.get_mut(current_cursor_index as u16, area.top());
            selected_cell.set_style(Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
        } else if current_cursor_index == max_index {
            let selected_input_row =  Rect::new(input_start_index, area.top(), self.length.clone() as u16, 1);
            buf.set_style(selected_input_row, Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
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

impl StatefulWidget for DropdownInputState {
    type State = DropdownInputState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default());

        let mut index: u16 = 0;
        let input_offset = area.left() + self.name.clone().len() as u16 + 1;
        if self.selected {
            if self.show_options {
                let selected_option = self.chosen_option.clone();
                for opt in self.options {
                    if opt == selected_option {
                        let selected_input_row = Rect::new(input_offset.clone(), area.top() + index.clone(), 12, 1);
                        log::info!("Selecting dropdown {} row {} : {}", self.name, index, self.chosen_option.clone());
                        buf.set_style(selected_input_row.clone(), Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
                    }
                    buf.set_string(input_offset, area.top() + index.clone(), opt.clone(), Style::default());
                    index += 1;
                }
            } else {
                buf.set_string(input_offset, area.top() + index.clone(), self.chosen_option.clone(), Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
            }
        } else {
            buf.set_string(input_offset, area.top() + index.clone(),self.chosen_option.clone(), Style::default());
        }

    }
}

