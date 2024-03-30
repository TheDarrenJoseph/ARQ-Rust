use tui::buffer::Cell;
use tui::style::Color;

use crate::character::Character;
use crate::global_flags;
use crate::map::objects::container::Container;
use crate::map::tile::TileDetails;
use crate::terminal::colour_mapper;

#[derive(Clone)]
#[derive(Debug)]
pub struct CellBuilder {
}

impl CellBuilder {
    pub fn from_tile(tile_details: &TileDetails) -> Cell {
        let symbol = tile_details.symbol.character.to_string();
        let fg = colour_mapper::map_colour(tile_details.symbol.colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        Cell { symbol, fg, bg, modifier }
    }

    pub fn from_character(character: &Character) -> Cell{
        let character_colour = character.get_colour();
        let symbol = character.get_symbol().to_string();
        let fg = colour_mapper::map_colour(character_colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        Cell { symbol, fg, bg, modifier }
    }

    pub fn from_container(container: &Container) -> Cell{
        let container_item = container.get_self_item();
        let symbol = container_item.symbol.character.to_string();
        let colour = container_item.symbol.colour;
        let fg = colour_mapper::map_colour(colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        Cell { symbol, fg, bg, modifier }
    }

    pub fn for_blank() -> Cell {
        if global_flags::GLOBALS.debugging_map_symbols {
            // For debugging - this makes the blanked area really obvious by using the block character
            Cell { symbol: String::from('\u{2588}'), fg: Color::Green, bg: Color::Black, modifier: tui::style::Modifier::empty() }
        } else {
            Cell { symbol: String::from(" "), fg: Color::Black, bg: Color::Black, modifier: tui::style::Modifier::empty() }
        }
    }
}