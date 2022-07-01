use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::widgets::StatefulWidget;

use crate::widget::{Widget, WidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct ButtonState {
    pub selected: bool,
    length: i8,
    name: String
}

pub fn build_button(length: i8, name: String) -> Widget {
    let name_input_state = WidgetType::Button( ButtonState { selected: false, length, name});
    Widget{ state_type: name_input_state}
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
