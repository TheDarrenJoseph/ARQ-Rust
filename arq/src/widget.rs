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
    selected: bool,
    length: i8,
    input : String,
    name: String,
    input_padding: i8,
    selected_index: i8,
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
        return self.input.clone();
    }
}

pub fn build_text_input(length: i8, name: String, input_padding: i8) -> Widget {
    let mut name_input_state = WidgetType::Text( TextInputState { selected: false, length, input: "".to_string(), name, input_padding,  selected_index: 0 });
    Widget{ state_type: name_input_state}
}

#[derive(Clone)]
pub struct DropdownInputState {
    pub selected: bool,
    show_options: bool,
    name: String,
    options : Vec<String>,
    selected_index: i8,
    chosen_option : String
}

pub fn build_dropdown(name: String, options: Vec<String>) -> Widget {
    let mut state = WidgetType::Dropdown( DropdownInputState { selected: false, show_options: false, name, selected_index: 0, chosen_option: options[0].to_string(), options});
    Widget{ state_type: state}
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

impl StatefulWidget for TextInputState {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {

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

impl StatefulWidget for DropdownInputState {
    type State = DropdownInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

#[cfg(test)]
mod text_text_input {
    use crate::widget::{Focusable, Widget, WidgetType, TextInputState, build_text_input};

    #[test]
    fn test_text_input_add_char() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        // WHEN we add a character
        match text_input.state_type {
            WidgetType::Text(mut state) => {
                state.add_char('A');

                // THEN we expect the widget state input to be "A"
                assert_eq!("A".to_string(),  state.input);
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_add_char_max_input() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        // WHEN we add 4 characters
        match text_input.state_type {
            WidgetType::Text(mut state) => {
                state.add_char('A');
                state.add_char('B');
                state.add_char('C');
                state.add_char('D');

                // THEN we expect the widget state input to be "ABC" and to have ignored the extra character
                assert_eq!("ABC".to_string(),  state.input);
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_delete_char() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        match text_input.state_type {
            WidgetType::Text(mut state) => {
                // AND we've adjusted it's input to be "A"
                state.input = "A".to_string();
                // WHEN we call to delete a char
                state.delete_char();
                // THEN we expect the widget state input to be ""
                assert_eq!("".to_string(),  state.input);
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_delete_char_empty_field() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        match text_input.state_type {
            WidgetType::Text(mut state) => {
                // WHEN we call to delete a char
                state.delete_char();
                // THEN we expect the widget state input to be ""
                assert_eq!("".to_string(),  state.input);
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_delete_char_many() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        match text_input.state_type {
            WidgetType::Text(mut state) => {
                // AND we've adjusted it's input to be "ABC"
                state.input = "ABC".to_string();
                // WHEN we call to delete 2 characters
                state.delete_char();
                state.delete_char();
                // THEN we expect the widget state input to be "A"
                assert_eq!("A".to_string(),  state.input);
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }
}

#[cfg(test)]
mod text_dropdown {
    use crate::widget::{Focusable, Widget, WidgetType, DropdownInputState, build_dropdown, build_text_input};

    #[test]
    fn test_dropdown_get_selection() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to get the initial selection
                // THEN we expect it to be "A"
                assert_eq!("A".to_string(),  state.get_selection());
                assert_eq!(false,  state.is_showing_options());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_toggle_show() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to toggle showing of options
                state.toggle_show();
                // THEN we expect it to be set to true
                assert_eq!(true,  state.is_showing_options());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_toggle_show_multi() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to toggle showing of options twice
                state.toggle_show();
                state.toggle_show();
                // THEN we expect it to be set to false again
                assert_eq!(false,  state.is_showing_options());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }


    #[test]
    fn test_dropdown_select_next() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to select the next item
                state.select_next();
                // THEN we expect the selection to be "B"
                assert_eq!("B".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_select_next_end_of_range() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to select the next item twice
                state.select_next();
                state.select_next();
                // THEN we expect the selection to be "B"
                assert_eq!("B".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_select_previous() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // AND we've selected the 2nd option
                state.select_next();
                assert_eq!("B".to_string(),  state.get_selection());
                // WHEN we call to select the next item
                state.select_previous();
                // THEN we expect the selection to be "A" (unchanged)
                assert_eq!("A".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_select_previous_end_of_range() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                assert_eq!("A".to_string(),  state.get_selection());
                // WHEN we call to select the next item
                state.select_previous();
                // THEN we expect the selection to be "A" (unchanged)
                assert_eq!("A".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

}
