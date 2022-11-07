use crate::widget::boolean_widget::BooleanState;
use crate::widget::button_widget::ButtonState;
use crate::widget::character_stat_line::CharacterStatLineState;
use crate::widget::console_input_widget::ConsoleInputState;
use crate::widget::dropdown_widget::DropdownInputState;
use crate::widget::number_widget::NumberInputState;
use crate::widget::text_widget::TextInputState;

pub mod text_widget;
pub mod dropdown_widget;
pub mod number_widget;
pub mod boolean_widget;
pub mod button_widget;
pub mod character_stat_line;
pub mod console_input_widget;
pub mod widgets;

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

// TODO potentially convert to a Widget trait/WidgetBase struct?
#[derive(Debug)]
pub enum WidgetType {
    Text(TextInputState),
    Console(ConsoleInputState),
    Number(NumberInputState),
    Dropdown(DropdownInputState),
    Button(ButtonState),
    Boolean(BooleanState),
    StatLine(CharacterStatLineState)
}

pub struct Widget {
    pub state_type: WidgetType
}

pub trait Named {
    fn get_name(&mut self) -> String;
}

impl Named for WidgetType {
    fn get_name(&mut self) -> String {
        match self {
            WidgetType::Text(state) => {
                state.get_name()
            },
            WidgetType::Boolean(state) => {
                state.get_name()
            },
            WidgetType::Console(_state) => {
                String::from("Console")
            },
            WidgetType::Number(state) => {
                state.get_name().clone()
            },
            WidgetType::Dropdown(state) => {
                state.get_name().clone()
            },
            WidgetType::Button(state) => {
                state.get_name()
            },
            _ => { "".to_string() }
        }
    }
}

pub trait Focusable {
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn is_focused(&mut self) -> bool;
}

impl Focusable for WidgetType {
    fn focus(&mut self) {
        match self {
            WidgetType::Text(state) =>  state.selected = true,
            WidgetType::Console(state) => state.selected = true,
            WidgetType::Number(state) => state.selected = true,
            WidgetType::Dropdown(state) => state.selected = true,
            WidgetType::Button(state) => state.selected = true,
            WidgetType::Boolean(state) => state.selected = true,
            _ => {}
        }
    }

    fn unfocus(&mut self) {
        match self {
            WidgetType::Text(state) =>  state.selected = false,
            WidgetType::Console(state) => state.selected = false,
            WidgetType::Number(state) => state.selected = false,
            WidgetType::Dropdown(state) => state.selected = false,
            WidgetType::Button(state) => state.selected = false,
            WidgetType::Boolean(state) => state.selected = false,
            _ => {}
        }
    }

    fn is_focused(&mut self) -> bool {
        match self {
            WidgetType::Text(state) => state.selected.clone(),
            WidgetType::Console(state) => state.selected.clone(),
            WidgetType::Number(state) => state.selected.clone(),
            WidgetType::Dropdown(state) => state.selected.clone(),
            WidgetType::Button(state) => state.selected.clone(),
            WidgetType::Boolean(state) => state.selected.clone(),
            _ => { false }
        }
    }
}
