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
use crate::engine::process::Progressible;

use crate::map::Map;
use crate::map::map_generator::MapGenerator;
use crate::progress::StepProgress;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::map_generation::MapGenerationFrameHandler;
use crate::view::util::progress_display::ProgressDisplay;

/*
    This joins MapGenerator and ProgressDisplay together asynchronously to display the map generation process with a progress bar
 */
pub struct MapGeneration<'rng, 'a, B : tui::backend::Backend> {
    pub map_generator: MapGenerator<'rng>,
    pub progress_display: ProgressDisplay<'a, B>
}

impl <B : tui::backend::Backend> MapGeneration<'_, '_, B> {
    pub(crate) async fn generate_level(&mut self) -> Result<Map, io::Error> {
        let progress = self.map_generator.get_progress().clone();
        let (tx, rx) = channel();
        let handling = self.progress_display.handle_progress(rx, progress.step_count);
        tx.send(self.map_generator.get_progress().clone());
        let map = self.map_generator.generate(tx);

        let result = join!(map, handling);
        return Ok(result.0);
    }
}