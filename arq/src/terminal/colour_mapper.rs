use crate::map::tile::Colour;

pub fn map_colour(tile_colour: Colour) -> ratatui::style::Color {
    match tile_colour {
        Colour::None => {
            ratatui::style::Color::Reset
        },
        Colour::Red => {
            ratatui::style::Color::Red
        },
        Colour::Green => {
            ratatui::style::Color::Green
        },
        Colour::Blue => {
            ratatui::style::Color::Blue
        },
        Colour::Cyan => {
            ratatui::style::Color::Cyan
        },
        Colour::Brown => {
            ratatui::style::Color::Rgb(181, 137, 0)
        }
        Colour::White => {
            ratatui::style::Color::White
        },
        Colour::Black => {
            ratatui::style::Color::Black
        }
    }
}
