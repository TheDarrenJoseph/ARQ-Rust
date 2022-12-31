use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::StatefulWidget;

use crate::widget::{StatefulWidgetState, StatefulWidgetType};


#[derive(Clone)]
#[derive(Debug)]
pub struct BooleanState {
    pub selected: bool,
    pub value : bool,
    length: i8,
    name: String
}

pub fn build_boolean_widget(length: i8, name: String, value: bool) -> StatefulWidgetState {
    let name_input_state = StatefulWidgetType::Boolean( BooleanState { selected: false, value, length, name});
    StatefulWidgetState { state_type: name_input_state }
}

impl BooleanState {
    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }
}

impl StatefulWidget for BooleanState {
    type State = BooleanState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let value = if self.value {
            String::from("ENABLED")
        } else {
            String::from("DISABLED")
        };

        let text = format!("{} {}", self.name.clone(), value);
        if self.selected {
            buf.set_string(area.left(), area.top(), text , Style::default().fg(Color::Red));
        } else {
            buf.set_string(area.left(), area.top(), text, Style::default());
        }
    }
}
