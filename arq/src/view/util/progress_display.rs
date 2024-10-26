use std::io;
use std::sync::mpsc::Receiver;

use termion::input::TermRead;

use crate::map::position::Area;
use crate::progress::MultiStepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui_areas::UI_AREA_NAME_MAIN;
use crate::ui::ui_areas_builder::UIAreasBuilder;
use crate::ui::ui_layout::LayoutType::SingleMainWindowCentered;
use crate::view::framehandler::map_generation::MapGenerationFrameHandler;
use crate::view::framehandler::{FrameData, FrameHandler};

pub struct ProgressDisplay<'a, B : ratatui::backend::Backend> {
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: MapGenerationFrameHandler
}

impl <B : ratatui::backend::Backend> ProgressDisplay<'_, B>  {
    pub async fn handle_progress(&mut self, rx : Receiver<MultiStepProgress>, total_steps: usize) {
        for _i in 0..=total_steps {
            let progress = rx.recv();
            if let Ok(p) = progress {
                self.show_progress(p.clone());
                if p.is_done() {
                    // Wait for confirmation
                    io::stdin().keys().next().unwrap().expect("The next keyboard key should have been captured");
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
                    .layout_type(SingleMainWindowCentered)
                    .build().1;

                let main_area = ui_areas.get_area(UI_AREA_NAME_MAIN).unwrap();
                fh.handle_frame(frame, FrameData { data: progress.clone(), ui_areas: ui_areas.clone(), frame_area: main_area.area })
            }).expect("The progress display should have been drawn.");
        }
    }
}