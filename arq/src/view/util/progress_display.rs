use std::io;
use std::sync::mpsc::Receiver;

use log::error;
use termion::input::TermRead;

use tui::layout::Rect;
use crate::map::position::Area;
use crate::progress::MultiStepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui_areas::{UI_AREA_NAME_MAIN, UIAreas};
use crate::ui::ui_areas_builder::UIAreasBuilder;
use crate::ui::ui_layout::LayoutType::{SINGLE_MAIN_WINDOW, SINGLE_MAIN_WINDOW_CENTERED};
use crate::ui::ui_util::{center_area};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::map_generation::MapGenerationFrameHandler;

pub struct ProgressDisplay<'a, B : tui::backend::Backend> {
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: MapGenerationFrameHandler
}

impl <B : tui::backend::Backend> ProgressDisplay<'_, B>  {
    pub async fn handle_progress(&mut self, rx : Receiver<MultiStepProgress>, total_steps: usize) {
        for _i in 0..=total_steps {
            let progress = rx.recv();
            if let Ok(p) = progress {
                self.show_progress(p.clone());
                if p.is_done() {
                    // Wait for confirmation
                    io::stdin().keys().next().unwrap();
                    return;
                }
            }
        }
    }

    fn show_progress(&mut self, progress: MultiStepProgress) {
        let current_step_number = progress.get_current_step_number();
        if let Some(step_number) = current_step_number {
            let step_count = progress.step_count();
            log::info!("Showing progress: {}/{}", step_number, step_count);
            let fh = &mut self.frame_handler;
            self.terminal_manager.terminal.draw(|frame| {
                let ui_areas= UIAreasBuilder::new(Area::from_rect(frame.size()))
                    .layout_type(SINGLE_MAIN_WINDOW_CENTERED)
                    .build().1;

                let main_area = ui_areas.get_area(UI_AREA_NAME_MAIN).unwrap();
                fh.handle_frame(frame, FrameData { data: progress.clone(), ui_areas: ui_areas.clone(), frame_area: main_area.area })
            });
        }
    }
}