use tui::buffer::Cell;
use std::time::Duration;

use crate::map::Map;
use crate::terminal_manager::TerminalManager;

pub struct MapView<'a, B : tui::backend::Backend> {
    pub map : Map<'a>,
    pub terminal_manager : &'a mut TerminalManager<B>
}

impl<B : tui::backend::Backend> MapView<'_, B>{
    pub fn draw_map(&mut self) {
        let backend = self.terminal_manager.terminal.backend_mut();
        backend.clear();

        let tiles = &self.map.tiles;
        let mut y : u16 = 0;
        for row in tiles {
            let mut x : u16 = 0;
            for tile_details in row {
                log::info!("Drawing position: {}, {}", x, y);

                let symbol = tile_details.symbol.to_string();
                let fg = tui::style::Color::Red;
                let bg = tui::style::Color::Black;
                let modifier = tui::style::Modifier::empty();
                let cell = Cell{ symbol, fg, bg,modifier};
                let cell_tup : (u16, u16, &Cell) = (x,y,&cell);

                let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                backend.draw(updates.into_iter());
                backend.flush();
                x += 1;
            }
            y += 1;
        }
        std::thread::sleep(Duration::from_millis(5000));
    }
}
