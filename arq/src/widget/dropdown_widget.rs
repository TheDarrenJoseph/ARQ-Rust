use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::widgets::StatefulWidget;

use crate::widget::{StatefulWidgetState, StatefulWidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct DropdownInputState {
    pub selected: bool,
    pub editable: bool,
    show_options: bool,
    name: String,
    options : Vec<String>,
    selected_index: i8,
    chosen_option : String
}

impl DropdownInputState {
    pub fn select(&mut self, input : String) {
        match self.options.iter().position(|o| *o == input) {
            Some(idx) => {
                self.selected_index = idx as i8;
                self.chosen_option = self.options[idx].clone();
            }, _ => {}
        }
    }

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

    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }

    pub fn toggle_show(&mut self) {
        self.show_options = !self.show_options.clone();
    }
}

pub fn build_dropdown(name: String, editable: bool, options: Vec<String>) -> StatefulWidgetState {
    let state = StatefulWidgetType::Dropdown( DropdownInputState { selected: false, editable, show_options: false, name, selected_index: 0, chosen_option: options[0].to_string(), options});
    StatefulWidgetState { state_type: state}
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