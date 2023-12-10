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
use crate::engine::level::Level;
use crate::map::Map;
use crate::map::objects::container::{Container, ContainerType};
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::map::tile::TileDetails;
use crate::terminal::colour_mapper;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::character_info_view::CharacterInfoView;
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::map_framehandler::{MapFrameHandler, MapFrameHandlerData};
use crate::view::framehandler::util::tabling::build_paragraph;
use crate::view::model::usage_line::{UsageCommand, UsageLine};

/*
    This view draws the following to the screen:
    1. Individual tiles of the map
    2. Characters
    3. Containers
 */
pub struct MapView<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub level : Level,
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
}

impl<B : tui::backend::Backend> View<bool> for MapView<'_, B> {

    fn begin(&mut self) -> Result<InputResult<bool>, Error> {
        self.draw(None)?;
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: true }, view_specific_result: None});
    }

    // There are 3 map areas to consider:
    // 1. Map Area - Map co-ords (the position/size of the actual map e.g tiles), this should currently always start at 0,0
    // 2. Map view area - View co-ords (The position/size of the map view relative to the entire terminal frame), this could start at 1,1 for example (accounting for borders)
    // 3. Map display area - Map co-ords (The position/size of the map 'viewfinder', the area that you can actually see the map through)
    // 3.1 The map display area is what will move with the character throughout larger maps
    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
        // Make use of the provided area if present
        if let Some(a) = area {
            self.view_area = Some(a);
            log::info!("Map view area height (y): {}", a.size_y);
        } else {
            log::info!("Map view area height unset");
        }

        let terminal = &mut self.terminal_manager.terminal;
        let mut map_framehandler = MapFrameHandler::new();
        let frame_size = self.view_area.unwrap().to_rect();
        let ui = &mut self.ui;

        // Frame handler data
        let level = self.level.clone();
        let map_display_area = self.map_display_area.clone();
        let data = MapFrameHandlerData { level, map_display_area};

        terminal.draw(|frame| {
            // First let the UI draw everything else
            ui.render(frame);

            // Then render the map
            let frame_data = FrameData { frame_size, data };
            map_framehandler.handle_frame(frame, frame_data);
        })
    }


}

impl <COM: tui::backend::Backend> InputHandler<bool> for MapView<'_, COM> {
    fn handle_input(&mut self, _input: Option<Key>) -> Result<InputResult<bool>, Error> {
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: None});
    }
}