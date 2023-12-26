
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span};
use tui::widgets::{Block, Gauge, Paragraph};



use crate::progress::StepProgress;



use crate::view::framehandler::{FrameData, FrameHandler};

pub struct MapGenerationFrameHandler {
    pub seed: String
}

fn build_gauge(progress: StepProgress) -> Gauge<'static> {
    let step_name = progress.step_name.clone();
    let label = format!("{}/{} : {}", progress.current_step, progress.step_count, step_name);
    return Gauge::default()
        .block(Block::default()
            .title("Map Generation"))
        .label(label)
        .gauge_style(Style::default().fg(Color::White).bg(Color::Black))
        .percent((100 / progress.step_count) * progress.current_step)
}


impl <B : tui::backend::Backend> FrameHandler<B, StepProgress> for MapGenerationFrameHandler {
    fn handle_frame(&mut self, frame: &mut Frame<B>, data: FrameData<StepProgress>) {
        let gauge = build_gauge(data.data);
        let area = data.frame_size;
        frame.render_widget(gauge, area);

        let seed = Span::from(String::from(format!("Map Seed: {}", self.seed)));
        let seed_area = Rect::new(0,0, frame.size().width, 1);
        frame.render_widget(Paragraph::new(seed), seed_area);
    }
}

