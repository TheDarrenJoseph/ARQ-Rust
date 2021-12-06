use crate::map::tile::{Colour};

pub fn map_colour(tile_colour: Colour) -> tui::style::Color {
    match tile_colour {
        Colour::None => {
            tui::style::Color::Reset
        },
        Colour::Red => {
            tui::style::Color::Red
        },
        Colour::Green => {
            tui::style::Color::Green
        },
        Colour::Blue => {
            tui::style::Color::Blue
        },
        Colour::Cyan => {
            tui::style::Color::Cyan
        },
        Colour::Brown => {
            tui::style::Color::Rgb(181, 137, 0)
        }
        Colour::White => {
            tui::style::Color::White
        },
        Colour::Black => {
            tui::style::Color::Black
        }
    }
}
