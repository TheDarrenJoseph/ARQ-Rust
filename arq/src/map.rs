use room::Room;

use crate::map::position::{Area, Position};
use crate::map::tile::{Tile, TileDetails};

pub mod objects;
pub mod map_generator;
pub mod position;
pub mod room;
pub mod tile;

#[derive(Clone)]
pub struct Map {
    pub area : Area,
    pub tiles : Vec<Vec<TileDetails>>,
    pub rooms : Vec<Room>
}

impl Map {

    pub fn get_tile(&self, position: Position) -> Option<TileDetails> {
        match self.tiles.get(position.y as usize) {
            Some (row) => {
                match row.get(position.x as usize) {
                    Some(details) => {
                        return Some(details.clone());
                    },
                    None => {
                        return None
                    }
                }
            },
            None => {
                return None
            }
        }
    }

    pub fn set_tile(&mut self, position: Position, tile: TileDetails) {
        let x = position.x as usize;
        let y = position.y as usize;
        self.tiles[y][x] = tile
    }

    pub fn get_rooms(&self) -> Vec<Room> {
        return self.rooms.clone();
    }

    pub fn is_paveable(&self, position: Position) -> bool {
        match self.get_tile(position) {
            Some(tile) => {
                // All traversible tile types are paveable, including NoTile
                if tile.tile_type == Tile::NoTile {
                    true
                } else {
                    tile.traversable
                }
            }, None => {
                false
            }
        }
    }

    pub fn is_traversible(&self, position: Position) -> bool {
        match self.get_tile(position) {
            Some(tile) => {
                tile.traversable
            }, None => {
                false
            }
        }
    }

    pub fn get_neighbors(&self, position: Position) -> Vec<Position> {

        let mut results = Vec::new();
        let area = self.area;
        if area.contains(position) {
            let neighbors = position.get_neighbors();
            for n in neighbors {
                if area.contains(n) {
                    results.push(n);
                }
            }

        }
        results
    }
}

#[cfg(test)]
mod tests {
    use crate::map::position::{build_square_area, Position};
    use crate::map::room::Room;
    use crate::map::tile::Tile;

    #[test]
    fn test_build_map() {
        let tile_library = crate::map::tile::build_library();
        assert_eq!(9, tile_library.len());

        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();

        let room_pos = Position { x: 0, y: 0 };
        let room_area = build_square_area(room_pos, 3);
        let doors = Vec::new();
        let room = Room { area: room_area, doors };

        let mut rooms = Vec::new();
        rooms.push(room);

        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);
        let map = crate::map::Map {
            area: map_area,
            tiles : vec![
                vec![ wall.clone(), wall.clone(), wall.clone() ],
                vec![ wall.clone(), rom.clone(), wall.clone() ],
                vec![ wall.clone(), wall.clone(), wall.clone() ],
        ], rooms
        };

        assert_eq!(3, map.tiles.len());
        assert_eq!(3, map.tiles[0].len());
        assert_eq!(3, map.tiles[1].len());
        assert_eq!(3, map.tiles[2].len());

    }

    #[test]
    fn test_adjust_map() {
        let tile_library = crate::map::tile::build_library();
        assert_eq!(9, tile_library.len());

        let wall = tile_library[&Tile::Wall].clone();

        let room_pos = Position { x: 0, y: 0 };
        let room_area = build_square_area(room_pos, 3);
        let doors = Vec::new();
        let room = Room { area: room_area, doors };

        let mut rooms = Vec::new();
        rooms.push(room);

        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);
        let mut map = crate::map::Map {
            area: map_area,
            tiles : vec![
                vec![ wall.clone(),  wall.clone(),  wall.clone()],
            ], rooms
        };

        assert_eq!(1, map.tiles.len());

        // WHEN we push an item to the first row
        map.tiles[0].push(wall.clone());
        // THEN we expect it to go from 3 to 4 items long
        assert_eq!(4, map.tiles[0].len());

        // THEN we expect it to be available at 0,1
        assert_eq!(crate::map::tile::Tile::Wall, map.tiles[0][1].tile_type);

        // AND WHEN we push an new row to the map
        map.tiles.push(vec![wall.clone()]);
        // THEN we expect the length to increase
        assert_eq!(1, map.tiles[1].len());
        // AND the new tile to be available at 1,0
        assert_eq!(crate::map::tile::Tile::Wall, map.tiles[1][0].tile_type);
    }
}
