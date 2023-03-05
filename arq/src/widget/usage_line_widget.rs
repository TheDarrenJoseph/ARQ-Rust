use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

use crate::character::character_details::CharacterDetails;
use crate::view::model::usage_line::UsageLine;
use crate::widget::{StatefulWidgetState, StatefulWidgetType};

impl Widget for UsageLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.describe();
        buf.set_string(area.x, area.y, line.as_str(), Style::default());
    }
}