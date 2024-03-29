use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::StatefulWidget;
use crate::ui::resolution::{Resolution};
use crate::view::MIN_RESOLUTION;

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

#[derive(Clone)]
pub struct DropdownOption<T> {
    pub display_name : &'static str,
    pub value: Option<T>
}

pub fn get_resolution_dropdown_options() -> Vec<DropdownOption<Resolution>> {
    let min_resolution_dropdown_option: DropdownOption<Resolution> = DropdownOption { display_name: "80x24", value: Some(MIN_RESOLUTION) };
    let fullscreen_dropdown_option: DropdownOption<Resolution> = DropdownOption { display_name: "FULLSCREEN", value: None };
    vec! [
        fullscreen_dropdown_option,
        min_resolution_dropdown_option
    ]
}

pub struct DropdownSetting<T> {
    pub(crate) options : Vec<T>,
    pub(crate) chosen_option : T
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
    let input_state = DropdownInputState {
        selected: false,
        editable,
        show_options: false, // TODO re-enable once the dropdown widget is fixed
        name,
        selected_index: 0,
        chosen_option: options[0].to_string(),
        options};

    let state = StatefulWidgetType::Dropdown(input_state);
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
                let style = Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::REVERSED | Modifier::UNDERLINED);
                buf.set_string(input_offset, area.top() + index.clone(), self.chosen_option.clone(), style);
            }
        } else {
            let style = Style::default()
                .fg(Color::DarkGray);
            buf.set_string(input_offset, area.top() + index.clone(),self.chosen_option.clone(), style);
        }

    }
}