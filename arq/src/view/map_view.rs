use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Error;
use log::{info, log};

use termion::event::Key;
use tui::buffer::Cell;
use tui::layout::Rect;
use tui::style::Color;

use crate::global_flags;
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


fn build_blanking_cell() -> Cell {
    if global_flags::GLOBALS.debugging_map_symbols {
        // For debugging - this makes the blanked area really obvious by using the block character
        Cell { symbol: String::from('\u{2588}'), fg: Color::Green, bg: Color::Black, modifier: tui::style::Modifier::empty() }
    } else {
        Cell { symbol: String::from(" "), fg: Color::Black, bg: Color::Black, modifier: tui::style::Modifier::empty() }
    }
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
    fn is_position_in_map_display_area(&self, position: Position) -> bool {
        self.map_display_area.contains_position(position)
    }

    fn is_position_in_view_area(&self, position: Position) -> bool {
        if let Some(view_area) = self.view_area {
            return view_area.contains(position.x, position.y)
        } else {
            false
        }
    }

    fn local_to_global(&self, view_area_position: Position) -> Option<Position> {
        if let Some(view_area) = self.view_area {
            // Globalise it to a map based co-ord
            let mut globalised_x = view_area_position.x - view_area.start_position.x;
            let mut globalised_y = view_area_position.y - view_area.start_position.y;

            // Further globalise it with the map display offset to get the true co-ordinate
            let display_area_start = self.map_display_area.start_position;
            globalised_x = display_area_start.x + globalised_x;
            globalised_y = display_area_start.y + globalised_y;

            let global_position = Position::new(globalised_x, globalised_y);
            return Some(global_position);
        }
        None
    }

     /*
     * Converts a global (i.e map relative) position to a local position for this view (one that can be used to render a character)
     */
    fn global_to_view_local(&self, global_position: Position) -> Option<Position> {
        if let Some(view_area) = self.view_area {
            let display_area_start = self.map_display_area.start_position;

            // Convert global position to local relative to the display area
            // As display area is also a global position, we can simply get the difference
            // i.e
            // if you're at global position {x: 2, y: 2}
            // and the display area starts at global pos: {x: 1, y: 1}
            // This relative value is {x: 1, y: 1} as you're 1,1 closer to 2,2
            let relative_global_x = global_position.x - display_area_start.x;
            let relative_global_y = global_position.y - display_area_start.y;

            // Apply the view position offsets to this
            // i.e
            // given {x: 1, y: 1}
            // and the view area starts at frame pos {x: 1, y: 1}
            // This gives {x: 2, y: 2}
            let local_x = relative_global_x + view_area.start_position.x;
            let local_y = relative_global_y + view_area.start_position.y;
            let view_position = Position::new(local_x, local_y);
            return Some(view_position);
        }
        None
    }

    fn draw_character(&mut self, character: &Character) -> Result<(), Error> {
        let character_colour = character.get_colour();
        let character_symbol = character.get_symbol();
        let fg = colour_mapper::map_colour(character_colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        let cell = Cell { symbol: character_symbol.to_string(), fg, bg, modifier };

        let global_character_position = character.get_global_position();

        if let Some(view_area) = self.view_area {
            // Check the player is in the displayed area
            if self.is_position_in_map_display_area(global_character_position) {
                let local_position = self.global_to_view_local(global_character_position);
                if let Some(view_position) = local_position {
                    if view_area.contains_position(view_position) {
                        let backend = self.terminal_manager.terminal.backend_mut();
                        let cell_tup: (u16, u16, &Cell) = (view_position.x, view_position.y, &cell);
                        let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                        backend.draw(updates.into_iter())?;
                        backend.flush()?;
                    }
                }

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
                for (global_container_position, container) in &self.map.containers {
                    // Check if the container position is in the current display area
                    if self.is_position_in_map_display_area(global_container_position.clone()) {
                        // Convert the global map co-ord to a local view co-ord
                        if let Some(view_position) = self.global_to_view_local(*global_container_position) {
                            if self.is_position_in_view_area(view_position) {
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
        let tile_pos_in_map_bounds = self.map.position_in_bounds(tile_position);
        let view_position_in_view_bounds = self.is_position_in_view_area(view_position);
        // The map display area is a map-coord based system, so we need to check the tile co-ord against it
        let tile_position_in_map_display_area = self.is_position_in_map_display_area(tile_position);
        if tile_pos_in_map_bounds && view_position_in_view_bounds && tile_position_in_map_display_area {
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
        Ok(())
    }

    fn clear_map_view(&mut self) -> Result<(), Error> {
        if let Some(view_area) = self.view_area {
            let blanking_cell: Cell = build_blanking_cell();
            // Clear everything in the view area (entire internal window area)
            for view_area_x in view_area.start_position.x..view_area.end_position.x {
                for view_area_y in view_area.start_position.y..view_area.end_position.y {
                    let cell_tup: (u16, u16, &Cell) = (view_area_x, view_area_y, &blanking_cell);
                    let updates: Vec<(u16, u16, &Cell)> = vec![cell_tup];
                    self.terminal_manager.terminal.backend_mut().draw(updates.into_iter())?;
                }
            }
            self.terminal_manager.terminal.backend_mut().flush();
            info!("Map view cleared");
        }
        Ok(())
    }

    // There are 3 map areas to consider:
    // 1. Map Area - Map co-ords (the position/size of the actual map e.g tiles), this should currently always start at 0,0
    // 2. Map view area - View co-ords (The position/size of the map view relative to the entire terminal frame), this could start at 1,1 for example (accounting for borders)
    // 3. Map display area - Map co-ords (The position/size of the map 'viewfinder', the area that you can actually see the map through)
    // 3.1 The map display area is what will move with the character throughout larger maps
    fn draw_map_tiles(&mut self) -> Result<(), Error> {
        if let Some(view_area) = self.view_area {
            let map_display_area = self.map_display_area;

            // For each co-ord in the map view area
            for view_area_x in view_area.start_position.x..view_area.end_position.x {
                for view_area_y in view_area.start_position.y..view_area.end_position.y {
                    let view_area_position = Position::new(view_area_x, view_area_y);
                    let globalised_position = self.local_to_global(view_area_position);
                    if let Some(global_pos) = globalised_position {
                        if (map_display_area.contains_position(global_pos)) {
                            let view_position = Position::new(view_area_x, view_area_y);
                            self.draw_tile(global_pos, view_position);
                            self.terminal_manager.terminal.backend_mut().flush();
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
        self.clear_map_view()?;
        self.draw_map_tiles()?;
        // TODO add back
        //log::info!("Drawing map (containers)");
        self.draw_containers()?;
        //log::info!("Drawing map (characters)");
        self.draw_characters()?;

        Ok(())
    }
}

impl <COM: tui::backend::Backend> InputHandler<bool> for MapView<'_, COM> {
    fn handle_input(&mut self, _input: Option<Key>) -> Result<InputResult<bool>, Error> {
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: None});
    }
}