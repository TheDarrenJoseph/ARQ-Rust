use std::io;
use std::sync::mpsc::channel;

use tokio::join;

use crate::engine::process::Progressible;
use crate::map::Map;
use crate::map::map_generator::MapGenerator;
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
        let handling = self.progress_display.handle_progress(rx, progress.step_count());
        tx.send(self.map_generator.get_progress().clone()).expect("Map generator progress should have been sent to the tx channel!");
        let map = self.map_generator.generate(tx);

        let result = join!(map, handling);
        return Ok(result.0);
    }
}