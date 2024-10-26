use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::StatefulWidget;

use crate::widget::{StatefulWidgetState, StatefulWidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct ButtonState {
    pub selected: bool,
    length: i8,
    name: String
}

pub fn build_button(length: i8, name: String) -> StatefulWidgetState {
    let name_input_state = StatefulWidgetType::Button( ButtonState { selected: false, length, name});
    StatefulWidgetState { state_type: name_input_state}
}

impl ButtonState {
    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }
}

impl StatefulWidget for ButtonState {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default());
        if self.selected {
            buf.set_string(area.left(), area.top(), self.name.clone(), Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED));
        }
    }
}
