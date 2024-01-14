use std::convert::TryInto;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span};
use tui::widgets::{Block, Gauge, Paragraph};



use crate::progress::{MultiStepProgress, Step};



use crate::view::framehandler::{FrameData, FrameHandler};

pub struct MapGenerationFrameHandler {
    pub seed: String
}

fn build_gauge(progress: MultiStepProgress) -> Option<Gauge<'static>> {
    let current_step_number = progress.get_current_step_number();
    if let Some(step_number) = current_step_number {
        let current_step: &Step = progress.get_current_step_value().unwrap();
        let step_name = current_step.description.clone();
        let step_count = progress.step_count();
        let progress_percentage = progress.get_progress_percentage();
        let label = format!("{}/{} : {}", step_number, step_count, step_name);
        return Some(Gauge::default()
            .block(Block::default()
                .title("Map Generation"))
            .label(label)
            .gauge_style(Style::default().fg(Color::White).bg(Color::Black))
            // No support for usize? Not ideal.
            .percent(progress_percentage.try_into().unwrap()))
    } else {
        return None;
    }
}


impl <B : tui::backend::Backend> FrameHandler<B,MultiStepProgress> for MapGenerationFrameHandler {
    fn handle_frame(&mut self, frame: &mut Frame<B>, data: FrameData<MultiStepProgress>) {
        let gauge_result = build_gauge(data.data);
        if let Some(gauge) = gauge_result {
            let area = data.frame_size;
            frame.render_widget(gauge, area);

            let seed = Span::from(String::from(format!("Map Seed: {}", self.seed)));
            let seed_area = Rect::new(0, 0, frame.size().width, 1);
            frame.render_widget(Paragraph::new(seed), seed_area);
        }
    }
}

