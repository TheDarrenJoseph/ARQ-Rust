use std::io;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use termion::raw::{RawTerminal};
use crate::map::Map;

pub struct MapView<'a, RT : std::io::Write> {
    map : Map<'a>,
    backend : tui::backend::TermionBackend<RT>
}

impl<RT : std::io::Write> MapView<'_, RT>{
    fn draw_map(&self) {
        let tiles = &self.map.tiles;
        let x = 0;
        for row in tiles {
            let y = 0;
            for tile_details in row {
                log::info!("Drawing position: {}, {}", x, y);
            }
        }
    }
}