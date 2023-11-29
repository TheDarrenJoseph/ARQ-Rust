use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Error;
use log::{info, log};

use termion::event::Key;
use tui::buffer::Cell;
use tui::layout::Rect;
use tui::style::Color;

use crate::character::Character;
use crate::character::characters::Characters;
use crate::map::Map;
use crate::map::objects::container::{Container, ContainerType};
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::map::tile::TileDetails;
use crate::terminal::colour_mapper;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::character_info_view::CharacterInfoView;
use crate::view::framehandler::util::tabling::build_paragraph;
use crate::view::model::usage_line::{UsageCommand, UsageLine};

/*
    This view draws the following to the screen:
    1. Individual tiles of the map
    2. Characters
    3. Containers

 */
pub struct MapView<'a, B : tui::backend::Backend> {
    pub map : &'a Map,
    pub ui : &'a mut UI,
    pub characters : Characters,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub view_area : Option<Area>, // Total view area
    pub map_display_area : Area // Possibly reduced display area
}

impl<B : tui::backend::Backend> MapView<'_, B> {

    /*
        Returns true if the given position is inside the range covered by this view
        e.g:
         GIVEN self start_position is x: 5, y: 5 (The map co-ordinate offset)
         AND self view_area is a size of 3 starting from x: 1, y: 1 (this offset is only relevant for display purposed)
         THEN an input Position x 6,7, or 8 would return true (5 + 3 = 8 so range 5-8)
         AND anything above 8 would return false
         AND anything below 5 would return false
     */
    fn is_position_in_map_range(&self, position: Position) -> bool {
        self.map_display_area.contains_position(position)
    }

    fn is_position_in_view(&self, position: Position) -> bool {
        if let Some(view_area) = self.view_area {
            return view_area.contains(position.x, position.y)
        } else {
            false
        }
    }

    /*
        Applies the starting map position offset to the given position
        The result of this is a position that refers to the map and not the view
     */
    fn apply_map_offset(&self, position: Position) -> Position {
        return Position { x: position.x + self.map_display_area.start_position.x, y: position.y + self.map_display_area.start_position.y }
    }

     /*
        Offsets a given Position by the view start position
        The result of this is a position that applies purely to the view
     */
    fn to_view_position(&self, position: Position) -> Option<Position> {
        if let Some(view_area) = self.view_area {
            let view_start = view_area.start_position;
            let position = Position { x: view_start.x + position.x, y: position.y + view_start.y };
            if self.is_position_in_view(position) {
                return Some(position);
            }
        }
        None
    }

    fn draw_character(&mut self, character: &Character) -> Result<(), Error> {
        let character_position = character.get_position();
        let character_colour = character.get_colour();
        let character_symbol = character.get_symbol();
        let fg = colour_mapper::map_colour(character_colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        let cell = Cell { symbol: character_symbol.to_string(), fg, bg, modifier };

        if self.is_position_in_map_range(character_position) {
            if let Some(view_position) = self.to_view_position(character_position) {
                let backend = self.terminal_manager.terminal.backend_mut();
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
        let player = characters.get_player().unwrap();
        self.draw_character(player);
        for npc in characters.get_npcs() {
            self.draw_character( npc);
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
                    if self.is_position_in_map_range(position.clone()) {
                        if let Some(view_position) = self.to_view_position(*position) {
                            if self.is_position_in_view(view_position) {
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
            }
        }
        Ok(())
    }

    fn get_tile(&self, tile_position: Position) -> &TileDetails {
        let tiles = &self.map.tiles.tiles;
        &tiles[tile_position.y as usize][tile_position.x as usize]
    }

    fn draw_tile(&mut self, tile_position: Position, view_position: Position) -> Result<(), Error> {
        if self.map.position_in_bounds(tile_position) && self.is_position_in_view(view_position) {
            if self.is_position_in_map_range(view_position) {
                let tile_details = self.get_tile(tile_position);
                let symbol = tile_details.symbol.character.to_string();
                let fg = colour_mapper::map_colour(tile_details.symbol.colour);
                let bg = tui::style::Color::Black;
                let modifier = tui::style::Modifier::empty();
                let cell = Cell { symbol, fg, bg, modifier };
                let cell_tup: (u16, u16, &Cell) = (view_position.x, view_position.y, &cell);
                let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                let backend = self.terminal_manager.terminal.backend_mut();
                backend.draw(updates.into_iter())?;
            }
        }
        Ok(())
    }

    fn draw_map_tiles(&mut self) -> Result<(), Error> {
        if let Some(view_area) = self.view_area {
            // Clear everything in the view area (entire internal window area)
            for view_area_x in view_area.start_position.x..view_area.end_position.x {
                for view_area_y in view_area.start_position.y..view_area.end_position.y  {
                    // For debugging - this makes the blanked area really obvious
                    //let cell = Cell { symbol: String::from("O"), fg: Color::Yellow, bg: Color::Black, modifier: tui::style::Modifier::empty() };
                    let cell = Cell { symbol: String::from(" "), fg: Color::Black, bg: Color::Black, modifier: tui::style::Modifier::empty() };
                    let cell_tup: (u16, u16, &Cell) = (view_area_x, view_area_y, &cell);
                    let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                    self.terminal_manager.terminal.backend_mut().draw(updates.into_iter())?;
                }
            }
            self.terminal_manager.terminal.backend_mut().flush();
            info!("Map view cleared");

            for view_area_x in 0..view_area.get_size_x() {
                for view_area_y in 0..view_area.get_size_y() {
                    let view_position = Position::new(view_area_x, view_area_y);
                    let offset_tile_position = self.apply_map_offset(view_position);

                    if self.is_position_in_map_range(offset_tile_position) {
                        if let Some(view_position) = self.to_view_position(offset_tile_position) {
                            self.draw_tile(offset_tile_position, view_position);
                        }
                    }
                }
            }
        }
        self.terminal_manager.terminal.backend_mut().flush();
        Ok(())
    }
}

impl<B : tui::backend::Backend> View<bool> for MapView<'_, B> {

    fn begin(&mut self) -> Result<InputResult<bool>, Error> {
        self.draw(None)?;
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: true }, view_specific_result: None});
    }

    /*
        This performs the following rendering operations:
        1. Calls to the base UI to render base components (Main window, Console window, stat bars)
        2. Draws the base tiles of the map (Walls, corridors, doors, etc)
     */
    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
        // Only use this if present
        if let Some(a) = area {
            self.view_area = Some(a);
            log::info!("Map view area height (y): {}", a.size_y);
        } else {
            log::info!("Map view area height unset");
        }

        // First let the UI draw everything else
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;



        log::info!("Drawing map (tiles)");
        self.draw_map_tiles()?;
        log::info!("Drawing map (containers)");
        self.draw_containers()?;
        log::info!("Drawing map (characters)");
        self.draw_characters()?;

        Ok(())
    }
}

impl <COM: tui::backend::Backend> InputHandler<bool> for MapView<'_, COM> {
    fn handle_input(&mut self, _input: Option<Key>) -> Result<InputResult<bool>, Error> {
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: None});
    }
}