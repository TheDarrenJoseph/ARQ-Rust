use crate::room::Room;
use crate::position::{Area};
use crate::tile::TileDetails;

pub struct Map <'a> {
    pub area : Area,
    pub tiles : Vec<Vec<&'a TileDetails>>,
    pub rooms : Vec<Room>
}

#[cfg(test)]
mod tests {
    use crate::container::ContainerType;
    use crate::tile::build_library;

    #[test]
    fn test_build_map() {
        let tile_library = crate::tile::build_library();
        assert_eq!(9, tile_library.len());

        let room = &tile_library[2];
        let wall = &tile_library[3];

        let room_pos = Position { x: 0, y: 0 };
        let room_area = build_area( room_pos, 3);
        let doors = Vec::new();
        let room = Room { room_area, doors };

        let rooms = Vec::new();
        rooms.push(room);

        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_area( map_pos, 3);
        let map = crate::map::Map {
            area: map_area,
            tiles : vec![
            vec![ wall,  wall,  wall],
            vec![ wall,  room,  wall],
            vec![ wall,  wall, wall]
        ], rooms
        };


        let mut map = crate::map::Map { tiles : vec![vec![no_tile_1; 1]; 1], rooms };
        assert_eq!(1, map.tiles.len());

        // WHEN we push an item to the first row
        map.tiles[0].push(no_tile_2);
        assert_eq!(2, map.tiles[0].len());

        // THEN we expect it to be available at 0,1
        assert_eq!(crate::tile::Tile::NoTile, map.tiles[0][1].tile_type);

        // AND WHEN we push an new row to the map
        map.tiles.push(vec![no_tile_3]);
        // THEN we expect the length to increase
        assert_eq!(1, map.tiles[1].len());
        // AND the new tile to be available at 1,0
        assert_eq!(crate::tile::Tile::NoTile, map.tiles[1][0].tile_type);
    }
}