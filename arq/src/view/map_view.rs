use tui::buffer::Cell;

use std::io::Error;
use crate::map::Map;
use crate::ui::{UI};
use crate::terminal::terminal_manager::TerminalManager;
use crate::terminal::colour_mapper;
use crate::character::Character;
use crate::view::View;

pub struct MapView<'a, B : tui::backend::Backend> {
    pub map : &'a Map,
    pub ui : &'a mut UI,
    pub characters : Vec<Character>,
    pub terminal_manager : &'a mut TerminalManager<B>
}

impl<B : tui::backend::Backend> MapView<'_, B>{
    pub fn draw_characters(&mut self) -> Result<(), Error> {
        log::info!("Drawing characters...");

        let backend = self.terminal_manager.terminal.backend_mut();
        for character in &self.characters {
            let position = character.get_position();
            let character_colour = character.get_colour();
            let fg = colour_mapper::map_colour(character_colour);
            let bg = tui::style::Color::Black;
            let modifier = tui::style::Modifier::empty();
            let cell = Cell{ symbol: "@".to_string(), fg, bg,modifier};
            let cell_tup : (u16, u16, &Cell) = (position.x+1,position.y+1,&cell);
            let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
            backend.draw(updates.into_iter())?;
            backend.flush()?;
        }
        Ok(())
    }
}

impl<B : tui::backend::Backend> View for MapView<'_, B>{
    fn draw(&mut self) -> Result<(), Error> {
        log::info!("Drawing map tiles...");

        let mut ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.render(frame) })?;

        let backend = self.terminal_manager.terminal.backend_mut();

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
                let cell_tup : (u16, u16, &Cell) = (x+1,y+1,&cell);

                let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                backend.draw(updates.into_iter())?;
                backend.flush()?;
            }
        }
        Ok(())
    }

    fn handle_input(&mut self) -> Result<bool, Error> {
        Ok(true)
    }
}