use tui::widgets::StatefulWidget;
use tui::layout::Rect;
use tui::buffer::Buffer;
use tui::style::{Color, Style, Modifier};

pub struct TextInput {
    pub name: String,
    pub input_padding: i8,
    pub length: i8,
    pub selected: bool,
    pub selected_index: i8
}

pub struct TextInputState {
    pub input : String
}

impl StatefulWidget for TextInput {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_string(area.left(), area.top(), self.name.clone(), Style::default().fg(Color::White).bg(Color::Black));
        for i in self.input_padding..self.length+self.input_padding {
            let name_length = self.name.len() as u16;
            let cell = buf.get_mut(area.left() + name_length + i as u16, area.top());
            cell.set_fg(tui::style::Color::Black);
            cell.set_bg(tui::style::Color::White);
            cell.set_symbol(" ");
        }
    }
}