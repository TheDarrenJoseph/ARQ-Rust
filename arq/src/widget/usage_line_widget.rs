use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::view::model::usage_line::UsageLine;

impl Widget for UsageLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.describe();
        buf.set_string(area.x, area.y, line.as_str(), Style::default());
    }
}