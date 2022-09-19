use std::io::Error;

use termion::event::Key;
use tui::buffer::Cell;
use tui::layout::Rect;

use crate::character::Character;
use crate::map::Map;
use crate::map::objects::container::{Container, ContainerType};
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::terminal::colour_mapper;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::UI;
use crate::view::{GenericInputResult, View};

pub struct MapView<'a, B : tui::backend::Backend> {
    pub map : &'a Map,
    pub ui : &'a mut UI,
    pub characters : Vec<Character>,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub view_area : Option<Area>
}

impl<B : tui::backend::Backend> MapView<'_, B>{
    pub fn draw_characters(&mut self) -> Result<(), Error> {
        log::info!("Drawing characters...");
        if let Some(view_area) = self.view_area {
            let view_start = view_area.start_position;

            let backend = self.terminal_manager.terminal.backend_mut();
            for character in &self.characters {
                let position = character.get_position();
                let character_colour = character.get_colour();
                let fg = colour_mapper::map_colour(character_colour);
                let bg = tui::style::Color::Black;
                let modifier = tui::style::Modifier::empty();
                let cell = Cell { symbol: "@".to_string(), fg, bg, modifier };
                let view_position = Position { x: view_start.x + position.x, y:  position.y + view_start.y};
                if view_area.contains_position(view_position) {
                    let cell_tup: (u16, u16, &Cell) = (view_position.x, view_position.y, &cell);
                    let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                    backend.draw(updates.into_iter())?;
                    backend.flush()?;
                }
            }
        }
        Ok(())
    }

    fn draw_container(&mut self, view_position: Position, container: &Container)  -> Result<(), Error> {
        let backend = self.terminal_manager.terminal.backend_mut();
        let container_item = container.get_self_item();
        let colour = container_item.colour;
        let fg = colour_mapper::map_colour(colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        let cell = Cell { symbol: container_item.symbol.to_string(), fg, bg, modifier };
        let cell_tup: (u16, u16, &Cell) = (view_position.x, view_position.y, &cell);
        let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
        backend.draw(updates.into_iter())?;
        backend.flush()?;
        Ok(())
    }

    pub fn draw_containers(&mut self) -> Result<(), Error> {
        log::info!("Drawing containers...");
        if let Some(view_area) = self.view_area {
            let view_start = view_area.start_position;

            if self.map.containers.is_empty() {
                log::error!("No containers exist in this map!");
            } else {
                for (position, container) in &self.map.containers {
                    let view_position = Position { x: view_start.x + position.x, y: position.y + view_start.y };
                    if view_area.contains_position(view_position) {
                        match container.container_type {
                            ContainerType::OBJECT => {
                                self.draw_container(view_position.clone(), container)?;
                            }
                            ContainerType::AREA => {
                                let item_count = container.get_total_count();
                                log::debug!("[map view] {} has {} items.", container.get_self_item().name, item_count);
                                if item_count > 0 {
                                    self.draw_container(view_position.clone(), container)?;
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw_cell(&mut self, x: u16, y: u16, cell_x: u16, cell_y: u16) -> Result<(), Error> {
        let view_area = self.view_area.unwrap();
        let backend = self.terminal_manager.terminal.backend_mut();
        let tiles = &self.map.tiles.tiles;
        if view_area.contains(cell_x, cell_y) && self.map.in_bounds(x as usize, y as usize) {
            let tile_details = &tiles[y as usize][x as usize];

            let symbol = tile_details.symbol.to_string();
            let fg = colour_mapper::map_colour(tile_details.colour);
            let bg = tui::style::Color::Black;
            let modifier = tui::style::Modifier::empty();
            let cell = Cell { symbol, fg, bg, modifier };
            let cell_tup: (u16, u16, &Cell) = (cell_x, cell_y, &cell);

            let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
            backend.draw(updates.into_iter())?;
            backend.flush()?;
        }
        Ok(())
    }

    fn draw_map_cells(&mut self) -> Result<(), Error> {
        let view_area = self.view_area.unwrap();
        let view_start_position = view_area.start_position;
        for x in 0..view_area.get_size_x() {
            for y in 0..view_area.get_size_y() {
                let cell_x = x + view_start_position.x as u16;
                let cell_y = y + view_start_position.y as u16;
                self.draw_cell(x, y, cell_x, cell_y)?
            }
        }
        Ok(())
    }
}

impl<B : tui::backend::Backend> View<'_, GenericInputResult> for MapView<'_, B> {

    fn begin(&mut self) -> Result<bool, Error> {
        self.draw(None)?;
        Ok(true)
    }

    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
        log::info!("Drawing map tiles...");

        let ui = &mut self.ui;
        let mut frame_size = Rect::new(0,0, 20, 20);
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
            frame_size = frame.size();
        })?;

        let view_area = if let Some(a) = area {
            a
        } else {
            let start_position = Position { x: frame_size.x + 1, y:frame_size.y + 1};
            build_rectangular_area(start_position, frame_size.width, frame_size.height)
        };
        self.view_area = Some(view_area);
        self.draw_map_cells()?;
        Ok(())
    }

    fn handle_input(&mut self, _input: Option<Key>) -> Result<bool, Error> {
        Ok(true)
    }
}