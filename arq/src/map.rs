use crate::tile::TileDetails;
use crate::items::Item;
use crate::container::Container;

pub struct Map <'a> {
    pub tiles : Vec<Vec<&'a TileDetails>>
}

#[cfg(test)]
mod tests {
    use crate::container::{ContainerType};
    use crate::tile::build_library;

    #[test]
    fn test_build_map() {
        let tile_library = crate::tile::build_library();
        assert_eq!(9, tile_library.len());

        let no_tile_1 = &tile_library[0];
        let no_tile_2 = &tile_library[0];
        let no_tile_3 = &tile_library[0];

        let mut map = crate::map::Map { tiles : vec![vec![no_tile_1; 1]; 1] };
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