use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::{Gauge, Widget};

struct LoadingScreen<'a> {
    gauge: Gauge<'a>
}

impl Widget for LoadingScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.gauge.render(area, buf);

        //buf.set_string(area.x, area.y, line.as_str(), Style::default());
    }
}