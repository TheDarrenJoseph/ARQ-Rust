use std::future::Future;
use std::io;
use std::time::Duration;
use futures::task;
use log::info;
use termion::input::TermRead;
use crate::map::Map;
use crate::map::map_generator::MapGenerator;
use crate::progress::StepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::map_generation::{MapGenerationFrameHandler};

pub struct MapGeneration<'a, 'rng, B : tui::backend::Backend> {
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub map_generator: MapGenerator<'rng>,
    pub frame_handler: MapGenerationFrameHandler
}

impl <B : tui::backend::Backend> MapGeneration<'_, '_, B> {

    fn get_progress(&self) -> StepProgress {
        self.map_generator.get_progress().clone()
    }

    fn update_progress(&mut self) {
        let mut progress = self.get_progress();
        log::info!("Showing progress: {}/{}", progress.current_step, progress.step_count);
        let fh = &mut self.frame_handler;
        self.terminal_manager.terminal.draw(|frame| {
            let mut area = frame.size().clone();
            area.y = frame.size().y / 2 + 2;
            area.x = area.width / 3;
            area.height = 4;
            area.width = area.width / 3;
            fh.handle_frame(frame, FrameData { data: progress.clone(), frame_size: area})
        });
    }

    pub(crate) async fn generate_level(&mut self) -> Result<Map, io::Error> {
        self.update_progress();
        let generator = &mut self.map_generator;
        let map_future = generator.generate();
        let map = map_future.await;
        // TODO make this update via polling
        self.update_progress();
        io::stdin().keys().next().unwrap()?;
        Ok(map)
    }
}