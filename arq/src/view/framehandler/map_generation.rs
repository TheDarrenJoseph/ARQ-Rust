use std::io::Error;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Gauge};
use crate::map::Map;
use crate::map::map_generator::MapGenerator;
use crate::map::position::Area;
use crate::progress::StepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::{GenericInputResult, InputResult, View};
use crate::view::framehandler::{FrameData, FrameHandler};

pub struct MapGenerationFrameHandler {
}

fn build_gauge(progress: StepProgress) -> Gauge<'static> {
    let step_name = progress.step_name.clone();
    let label = format!("{}/{} : {}", progress.current_step, progress.step_count, step_name);
    return Gauge::default()
        .block(Block::default()
            .title("Map Generation"))
        .label(label)
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent((100 / progress.step_count) * progress.current_step)
}


impl <B : tui::backend::Backend> FrameHandler<B, StepProgress> for MapGenerationFrameHandler {
    fn handle_frame(&mut self, frame: &mut Frame<B>, data: FrameData<StepProgress>) {
        let gauge = build_gauge(data.data);
        let area = data.frame_size;
        frame.render_widget(gauge, area);
    }
}

