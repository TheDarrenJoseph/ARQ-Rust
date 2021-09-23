use crate::room::Room;
use crate::position::{Area};
use crate::tile::TileDetails;

pub struct Map {
    pub area : Area,
    pub tiles : Vec<Vec<TileDetails>>,
    pub rooms : Vec<Room>
}

#[cfg(test)]
mod tests {
    use crate::container::ContainerType;
    use crate::tile::{Tile, build_library};
    use crate::room::Room;
    use crate::position::{Position, Area, build_square_area};

    #[test]
    fn test_build_map() {
        let tile_library = crate::tile::build_library();
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
        let tile_library = crate::tile::build_library();
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
        assert_eq!(crate::tile::Tile::Wall, map.tiles[0][1].tile_type);

        // AND WHEN we push an new row to the map
        map.tiles.push(vec![wall.clone()]);
        // THEN we expect the length to increase
        assert_eq!(1, map.tiles[1].len());
        // AND the new tile to be available at 1,0
        assert_eq!(crate::tile::Tile::Wall, map.tiles[1][0].tile_type);
    }
}