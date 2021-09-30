use rand::Rng;
use std::collections::HashMap;
use crate::map::Map;
use crate::room::Room;
use crate::door::{build_door};
use crate::position::{Position, Area, build_square_area, Side};
use crate::tile::{Tile, TileDetails, build_library};

pub struct MapGenerator {
    min_room_size: u16,
    max_room_size: u16,
    room_area_quota_percentage: u16,
    room_area_percentage: u16,
    max_door_count: u16,
    tile_library :  HashMap<Tile, TileDetails>,
    map_area : Area,
    taken_positions : Vec<Position>,
    possible_room_positions : Vec<Position>
}

pub fn build_generator(map_area : Area) -> MapGenerator {
    MapGenerator { min_room_size: 3, max_room_size: 6,
        room_area_quota_percentage: 30, room_area_percentage: 0, max_door_count: 4,
        tile_library: build_library(), map_area, taken_positions: Vec::new(),
        possible_room_positions : Vec::new()}
}

impl MapGenerator {
    pub fn generate(&mut self) -> Map {
        let mut map = self.build_map();
        map = self.add_rooms_to_map(map);
        map
    }

    fn generate_room(&self, room_pos : Position, size: u16) -> Room {
        let room_area = build_square_area( room_pos, size);
        let mut room = Room { area: room_area, doors: Vec::new() };

        let mut chosen_sides = Vec::<Side>::new();
        let room_sides = room.get_sides();

        let mut doors = Vec::new();
        let mut rng = rand::thread_rng();
        let door_count = rng.gen_range(1..=self.max_door_count);
        let map_area = self.map_area.clone();
        let map_sides = map_area.get_sides();

        for _x in 0..door_count {
            let side : Side = rand::random();
            if !chosen_sides.contains(&side) {
                chosen_sides.push(side);

                let side_idx = room_sides.iter().position(|area_side| area_side.side == side);
                let area_side = room_sides.get(side_idx.unwrap() as usize).unwrap();
                let door_position = area_side.get_mid_point();

                // Don't allow doors at the edges of the map
                for map_side in &map_sides {
                    if map_side.area.contains(door_position) {
                        continue;
                    }
                }

                let door = build_door(door_position);
                doors.push(door);
            }
        }
        room.doors = doors;
        room
    }

    fn remove_possible_position(&mut self, position : Position) -> Option<Position>{
        let room_pos_idx = &self.possible_room_positions.iter().position(|pos| *pos == position);
        match room_pos_idx {
            Some(idx) => {
                self.possible_room_positions.remove(*idx);
                self.taken_positions.push(position);
                Some(position)
            },
            None => {
                None
            }
        }
    }

    fn generate_rooms(&mut self) -> Vec<Room> {

        let total_area = self.map_area.get_total_area();
        let mut room_area_total = 0;
        let mut remaining_area_total = total_area - room_area_total;
        let mut total_area_usage_percentage : u16 = 0;

        let mut rooms : Vec<Room> = Vec::new();
        let mut rng = rand::thread_rng();
        self.find_possible_room_positions();
        while total_area_usage_percentage < self.room_area_quota_percentage && self.possible_room_positions.len() > 0 {
            let random_pos = rng.gen_range(0..self.possible_room_positions.len());
            let position = *self.possible_room_positions.get(random_pos).unwrap();

            // Try each position 3 times with a different size
            for _x in 0..=2 {
                let size = rng.gen_range(self.min_room_size..=self.max_room_size);
                let potential_area = build_square_area(position, size);
                let mut position_taken = false;
                for r in &rooms {
                    let taken_area = r.area;
                    if taken_area.intersects_or_touches(potential_area.clone()) {
                        log::info!("Cannot fit, intersection of proposed Start:{},{}, End:{},{} itersects: {},{}..{},{}",
                            potential_area.start_position.x, potential_area.start_position.y, potential_area.end_position.x,potential_area.end_position.y,
                            taken_area.start_position.x, taken_area.start_position.y, taken_area.end_position.x,taken_area.end_position.y);
                        position_taken = true;
                    }
                }


                if self.map_area.can_fit(position, size) && !position_taken && room_area_total < total_area {
                    let room = self.generate_room(position, size);
                    let room_positions = room.area.get_positions();
                    let room_area = room.area.get_total_area();
                    log::info!("New room with area: {}", room_area);
                    room_area_total += room_area;

                    for taken_pos in &room_positions {
                        self.remove_possible_position(*taken_pos);
                    }

                    rooms.push(room);
                    if remaining_area_total >= room_area_total {
                        remaining_area_total -= room_area_total;
                    }

                    let area_usage_percentage : u16 = (room_area as f32 / total_area as f32 * 100.00 as f32) as u16;
                    log::info!("Room area usage: {}%", area_usage_percentage);
                    total_area_usage_percentage += area_usage_percentage;
                    log::info!("Total room area usage: {}/{}", room_area_total, total_area);
                    log::info!("Total room area usage: {}%", total_area_usage_percentage);
                    break;
                }
            }
        }
        rooms
    }

    fn find_possible_room_positions(&mut self) {
        let map_end = self.map_area.end_position;
        // +1/-1 to allow outer boundaries
        for x in 1..map_end.x - 1  {
            for y in 1..map_end.y - 1 {
                let position = Position { x, y };
                self.possible_room_positions.push(position);
            }
        }
    }

    fn build_empty_tiles(&mut self) -> Vec<Vec<TileDetails>> {
        // TODO hash map the library entries we we can lookup by NoTile, etc
        let mut map_tiles = Vec::new();
        let mut row;
        for y in self.map_area.start_position.y..=self.map_area.end_position.y {
            row = Vec::new();
            for x in self.map_area.start_position.x..=self.map_area.end_position.x {
                log::info!("New tile at: {}, {}", x,y);
                row.push( self.tile_library[&Tile::NoTile].clone());
            }
            map_tiles.push(row);
        }
        map_tiles
    }

    fn add_rooms_to_map(&mut self, mut map: Map) -> Map {
        let tile_library = crate::tile::build_library();
        let room_tile = &tile_library[&Tile::Room].clone();
        let wall_tile = &tile_library[&Tile::Wall].clone();

        let rooms = &map.rooms;
        for room in rooms {
            let inside_area = room.get_inside_area();
            let inside_positions = inside_area.get_positions();
            for position in inside_positions {
                let x = position.x as usize;
                let y = position.y as usize;
                //log::info!("Room tile at: {}, {}", x,y);
                map.tiles[y][x] = room_tile.clone();
            }

            let sides = room.get_sides();
            for side in sides {
                let side_positions = side.area.get_positions();
                for position in side_positions {
                    let x = position.x as usize;
                    let y = position.y as usize;
                    //log::info!("Wall tile at: {}, {}", x,y);
                    map.tiles[y][x] = wall_tile.clone();
                }
            }

            let doors = &room.doors;
            for door in doors {
                let position = door.position;
                let x = position.x as usize;
                let y = position.y as usize;
                //log::info!("Door tile at: {}, {}", x,y);
                map.tiles[y][x] = door.tile_details.clone();
            }
        }
        map
    }

    fn build_map(&mut self) -> Map {
        let rooms = self.generate_rooms();
        let map_area = self.map_area.clone();
        let map_tiles = self.build_empty_tiles();
        return crate::map::Map {
            area: map_area,
            tiles : map_tiles,
            rooms
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::position::{Position, build_square_area};
    use crate::map_generator::{build_generator};

    #[test]
    fn test_build_generator() {
        // GIVEN a 12x12 map board
        let map_area = build_square_area(Position {x: 0, y: 0}, 12);
        let generator = build_generator(map_area);

        assert_eq!(3, generator.min_room_size);
        assert_eq!(6, generator.max_room_size);
        assert_eq!(40, generator.room_area_quota_percentage);
        assert_eq!(0, generator.room_area_percentage);
        assert_eq!(4, generator.max_door_count);
        assert!(generator.tile_library.len() > 0);
        assert_eq!(map_area, generator.map_area);
        assert_eq!(0, generator.taken_positions.len());
        assert_eq!(0, generator.possible_room_positions.len());
    }

    #[test]
    fn test_generate_room() {
        let map_area = build_square_area(Position {x: 0, y: 0}, 12);
        let generator = build_generator(map_area);

        let room = generator.generate_room(Position {x: 0, y: 0}, 3);
        let expected_area = build_square_area(Position {x: 0, y: 0}, 3);
        assert_eq!(expected_area, room.area);
        assert!(!room.doors.is_empty());
    }

    #[test]
    fn test_generate_rooms() {
        let map_size = 12;
        let map_area = build_square_area(Position {x: 0, y: 0}, map_size);
        let mut generator = build_generator(map_area);
        let rooms = generator.generate_rooms();
        assert_ne!(0, rooms.len());

        for room in rooms {
            let area = room.area;
            let start_pos = area.start_position;
            assert!(start_pos.x <= 12 && start_pos.y < 12, "Expected room start position < 12 for x,y, but was: {}, {}", start_pos.x, start_pos.y);
            let end_pos = area.end_position;
            assert!(end_pos.x <= 12 && end_pos.y < 12, "Expected room end position < 12 for x,y, but was: {}, {}", end_pos.x, end_pos.y);
        }
    }

    #[test]
    fn test_generate() {
        let map_size = 12;
        let map_area = build_square_area(Position {x: 0, y: 0}, map_size);
        let mut generator = build_generator(map_area);
        let map = generator.generate();

        let area = map.area;
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(11, area.end_position.x);
        assert_eq!(11, area.end_position.y);

        let tiles = map.tiles;
        assert_eq!(12, tiles.len());
        for row in tiles {
            assert_eq!(12, row.len());
        }

        let rooms = map.rooms;
        assert_ne!(0, rooms.len());
        for room in rooms {
            let area = room.area;
            let start_pos = area.start_position;
            assert!(start_pos.x <= 12 && start_pos.y < 12, "Expected room start position < 12 for x,y, but was: {}, {}", start_pos.x, start_pos.y);
            let end_pos = area.end_position;
            assert!(end_pos.x <= 12 && end_pos.y < 12, "Expected room end position < 12 for x,y, but was: {}, {}", end_pos.x, end_pos.y);
        }
    }
}