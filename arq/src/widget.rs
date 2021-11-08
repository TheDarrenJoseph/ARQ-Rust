pub mod text_widget;
pub mod dropdown_widget;
pub mod number_widget;
pub mod button_widget;

use tui::widgets::StatefulWidget;
use tui::layout::Rect;
use tui::buffer::Buffer;
use tui::style::{Style, Modifier};

use crate::widget::text_widget::TextInputState;
use crate::widget::dropdown_widget::DropdownInputState;
use crate::widget::number_widget::NumberInputState;
use crate::widget::button_widget::ButtonState;

pub fn build_buffer(length: i8, input: String) -> String {
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
    Button(ButtonState)
}

pub struct Widget {
    pub state_type: WidgetType
}

pub trait Focusable {
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn is_focused(&mut self) -> bool;
}

impl Focusable for WidgetType {
    fn focus(&mut self) {
        match self {
            WidgetType::Text(state) => {
                state.selected = true;
            },
            WidgetType::Number(state) => {
                state.selected = true;
            },
            WidgetType::Dropdown(state) => {
                state.selected = true;
            },
            WidgetType::Button(state) => {
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
            },
            WidgetType::Dropdown(state) => {
                state.selected = false;
            },
            WidgetType::Button(state) => {
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
            },
            WidgetType::Button(state) => {
                state.selected.clone()
            },
        }
    }
}
