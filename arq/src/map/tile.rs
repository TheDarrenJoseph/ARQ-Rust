use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Tile
{
    NoTile,Corridor,Room,Wall,Window,Door,Entry,Exit,Deadly
}

#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq, Debug)]
pub enum Colour {None,Red,Green,Blue,Cyan,Brown,White,Black}

#[derive(Clone)]
pub struct TileDetails
{
    id: u64,
    pub tile_type: Tile,
    pub traversable: bool,
    pub symbol: char,
    pub colour: Colour,
    pub name: String
}

pub fn build_library() -> HashMap<Tile, TileDetails> {
    let tile_details = [
        TileDetails {id: 0,     tile_type:  Tile::NoTile,   traversable: false, symbol: ' ', colour: Colour::None, name:  "Empty".to_string()},
        TileDetails {id: 1,     tile_type:  Tile::Corridor, traversable: true, symbol: '-', colour: Colour::Blue, name:  "Corridor".to_string()},
        TileDetails {id: 2,     tile_type:  Tile::Room,     traversable: true, symbol: '-', colour: Colour::Blue, name:  "Room".to_string()},
        TileDetails {id: 3,     tile_type:  Tile::Wall,     traversable: false, symbol: '#', colour: Colour::Brown, name:  "Wall".to_string()},
        TileDetails {id: 4,     tile_type:  Tile::Window,   traversable: false, symbol: '%', colour: Colour::Cyan, name:  "Window".to_string()},
        TileDetails {id: 5,     tile_type:  Tile::Door,     traversable: true, symbol: '=', colour: Colour::White, name:  "Door".to_string()},
        TileDetails {id: 6,     tile_type:  Tile::Entry,    traversable: true, symbol: '^', colour: Colour::Green, name:  "Entry".to_string()},
        TileDetails {id: 7,     tile_type:  Tile::Exit,     traversable: true, symbol: '^', colour: Colour::Red, name:  "Exit".to_string()},
        TileDetails {id: 8,     tile_type:  Tile::Deadly,   traversable: false, symbol: '!', colour: Colour::Red, name:  "Deadly".to_string()}
    ];

    let mut tile_map = HashMap::new();
    for details in tile_details.iter() {
        tile_map.insert(details.tile_type, details.clone());
    }
    tile_map
}

#[cfg(test)]
mod tests {
    use crate::map::tile::build_library;

    #[test]
    fn test_build_library() {
        let library = build_library();
        assert_eq!(9, library.len());
    }
}