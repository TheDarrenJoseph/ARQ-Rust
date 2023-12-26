

use std::sync::mpsc::Receiver;

use log::error;

use tui::layout::Rect;
use crate::progress::StepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui_util::{center_area};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::map_generation::MapGenerationFrameHandler;

pub struct ProgressDisplay<'a, B : tui::backend::Backend> {
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: MapGenerationFrameHandler
}

impl <B : tui::backend::Backend> ProgressDisplay<'_, B>  {
    pub async fn handle_progress(&mut self, rx : Receiver<StepProgress>, total_steps: u16) {
        for _i in 0..=total_steps {
            let progress = rx.recv();
            if let Ok(p) = progress {
                self.show_progress(p.clone());
                if p.is_done() {
                    // Wait for confirmation
                    //io::stdin().keys().next().unwrap();
                    return;
                }
            }
        }
    }

    fn show_progress(&mut self, progress: StepProgress) {
        log::info!("Showing progress: {}/{}", progress.current_step, progress.step_count);
        let fh = &mut self.frame_handler;
        self.terminal_manager.terminal.draw(|frame| {
            let target_area = Rect::new(0,0, 45,6);
            let area_result = center_area(target_area, frame.size(), target_area);

            if let Ok(area) = area_result {
                fh.handle_frame(frame, FrameData { data: progress.clone(), frame_size: area})
            } else {
                error!("{}", area_result.err().unwrap());
            }
        });
    }
}