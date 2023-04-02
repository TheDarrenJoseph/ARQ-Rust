use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Error;

use termion::event::Key;
use tui::buffer::Cell;
use tui::layout::Rect;

use crate::character::Character;
use crate::character::characters::Characters;
use crate::map::Map;
use crate::map::objects::container::{Container, ContainerType};
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::terminal::colour_mapper;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::character_info::CharacterInfoView;
use crate::view::framehandler::util::tabling::build_paragraph;
use crate::view::model::usage_line::{UsageCommand, UsageLine};

/*
    This View is responsible for showing the "default" view while playing including:
    1. The current map/level and it's contents
    2. The console
 */
pub struct MapView<'a, B : tui::backend::Backend> {
    pub map : &'a Map,
    pub ui : &'a mut UI,
    pub characters : Characters,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub view_area : Option<Area>
}

impl<B : tui::backend::Backend> MapView<'_, B>{
    fn draw_character(&mut self, character: &Character) -> Result<(), Error> {
        if let Some(view_area) = self.view_area {
            let view_start = view_area.start_position;
            let backend = self.terminal_manager.terminal.backend_mut();

            let position = character.get_position();
            let character_colour = character.get_colour();
            let character_symbol = character.get_symbol();
            let fg = colour_mapper::map_colour(character_colour);
            let bg = tui::style::Color::Black;
            let modifier = tui::style::Modifier::empty();
            let cell = Cell { symbol: character_symbol.to_string(), fg, bg, modifier };
            let view_position = Position { x: view_start.x + position.x, y: position.y + view_start.y };
            if view_area.contains_position(view_position) {
                let cell_tup: (u16, u16, &Cell) = (view_position.x, view_position.y, &cell);
                let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                backend.draw(updates.into_iter())?;
                backend.flush()?;
            }
        }
        Ok(())
    }

    pub fn draw_characters(&mut self) -> Result<(), Error> {
        log::info!("Drawing characters...");
        let characters = &mut self.characters.clone();
        if let Some(view_area) = self.view_area {
            let player = characters.get_player().unwrap();
            self.draw_character(player);
            for npc in characters.get_npcs() {
                self.draw_character( npc);
            }
        }
        Ok(())
    }

    fn draw_container(&mut self, view_position: Position, container: &Container)  -> Result<(), Error> {
        let backend = self.terminal_manager.terminal.backend_mut();
        let container_item = container.get_self_item();
        let colour = container_item.symbol.colour;
        let fg = colour_mapper::map_colour(colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        let cell = Cell { symbol: container_item.symbol.character.to_string(), fg, bg, modifier };
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
                                log::debug!("[map view] {} has {} items.", container.get_self_item().get_name(), item_count);
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

            let symbol = tile_details.symbol.character.to_string();
            let fg = colour_mapper::map_colour(tile_details.symbol.colour);
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

impl<B : tui::backend::Backend> View<bool> for MapView<'_, B> {

    fn begin(&mut self) -> Result<InputResult<bool>, Error> {
        self.draw(None)?;
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: true }, view_specific_result: None});
    }

    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
        log::info!("Drawing map...");

        let ui = &mut self.ui;

        self.view_area = area;
        let map_area = self.view_area.unwrap();

        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;

        self.draw_map_cells()?;

        Ok(())
    }
}

impl <COM: tui::backend::Backend> InputHandler<bool> for MapView<'_, COM> {
    fn handle_input(&mut self, _input: Option<Key>) -> Result<InputResult<bool>, Error> {
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: None});
    }
}