use std::{io, thread};
use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver};
use std::task::{Context, Poll};
use std::time::Duration;

use futures::future::join;
use futures::task;
use log::info;
use termion::input::TermRead;
use tokio::join;

use crate::map::Map;
use crate::map::map_generator::MapGenerator;
use crate::progress::StepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::map_generation::MapGenerationFrameHandler;

pub struct MapGeneration<'rng, 'a, B : tui::backend::Backend> {
    pub map_generator: MapGenerator<'rng>,
    pub progress_display: ProgressDisplay<'a, B>
}

pub struct ProgressDisplay<'a, B : tui::backend::Backend> {
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: MapGenerationFrameHandler
}

impl <B : tui::backend::Backend> ProgressDisplay<'_, B>  {
    async fn handle_progress(&mut self, rx : Receiver<StepProgress>) {
        loop {
            let mut progress = rx.recv();
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

    fn show_progress(&mut self, progress: StepProgress) {
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
}

impl <B : tui::backend::Backend> MapGeneration<'_, '_, B> {
    pub(crate) async fn generate_level(&mut self) -> Result<Map, io::Error> {
        let (tx, rx) = channel();
        let handling = self.progress_display.handle_progress(rx);
        tx.send(self.map_generator.get_progress().clone());
        let map = self.map_generator.generate(tx);

        let result = join!(map, handling);
        return Ok(result.0);
    }
}