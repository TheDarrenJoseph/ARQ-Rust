use tui::buffer::Cell;
use std::time::Duration;

use crate::io::Error;
use crate::map::Map;
use crate::terminal_manager::TerminalManager;
use crate::colour_mapper;

pub struct MapView<'a, B : tui::backend::Backend> {
    pub map : &'a Map,
    pub terminal_manager : &'a mut TerminalManager<B>
}

impl<B : tui::backend::Backend> MapView<'_, B>{
    pub fn draw_map(&mut self) -> Result<(), Error> {
        let backend = self.terminal_manager.terminal.backend_mut();
        backend.clear()?;

        let start_position = self.map.area.start_position;
        let end_position =  self.map.area.end_position;

        let tiles = &self.map.tiles;
        let start_x = start_position.x;
        let start_y = start_position.y;
        let end_x = end_position.x;
        let end_y = end_position.y;
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                //log::info!("Drawing a position: {}, {}", x, y);

                let tile_details = &tiles[usize::from(y)][usize::from(x)];

                let symbol = tile_details.symbol.to_string();
                let fg = colour_mapper::map_colour(tile_details.colour);
                let bg = tui::style::Color::Black;
                let modifier = tui::style::Modifier::empty();
                let cell = Cell{ symbol, fg, bg,modifier};
                let cell_tup : (u16, u16, &Cell) = (x,y,&cell);

                let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                backend.draw(updates.into_iter())?;
                backend.flush()?;
            }
        }
        Ok(())
    }
}
