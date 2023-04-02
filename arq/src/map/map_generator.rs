use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::Sender;
use std::task::{Context, Poll};
use std::time::Duration;
use rand::distributions::Standard;
use rand::Rng;

use rand_pcg::Pcg64;
use uuid::Uuid;

use crate::engine::pathfinding::Pathfinding;
use crate::engine::process::Progressible;
use crate::map::{Map, Tiles};
use crate::map::objects::{container, items};
use crate::map::objects::container::{Container, ContainerType};
use crate::map::objects::door::build_door;
use crate::map::objects::items::{Item, ItemForm, MaterialType};
use crate::map::position::{Area, build_square_area, Position, Side};
use crate::map::room::{build_room, Room};
use crate::map::tile::{build_library, Tile, TileDetails};
use crate::map::tile::Tile::NoTile;
use crate::progress::StepProgress;

pub struct MapGenerator<'rng> {
    min_room_size: u16,
    max_room_size: u16,
    room_area_quota_percentage: u16,
    max_door_count: u16,
    tile_library :  HashMap<Tile, TileDetails>,
    map_area : Area,
    taken_positions : Vec<Position>,
    possible_room_positions : Vec<Position>,
    rng: &'rng mut Pcg64,
    pub progress: StepProgress,
    map: Map
}

pub fn build_generator<'a>(rng : &'a mut Pcg64, map_area : Area) -> MapGenerator<'a> {
    let progress = StepProgress { step_name: "".to_string(), current_step: 0, step_count: 6};
    MapGenerator { min_room_size: 3, max_room_size: 6,
        room_area_quota_percentage: 30, max_door_count: 4,
        tile_library: build_library(), map_area, taken_positions: Vec::new(),
        possible_room_positions : Vec::new(),
        rng,
        progress,
        map: Map {area: map_area, tiles: Tiles { tiles: Vec::new() }, rooms: Vec::new(), containers: HashMap::new()}}
}

pub fn build_dev_player_inventory() -> Container {
    let mut container = container::build(Uuid::new_v4(), "Player's Inventory".to_owned(), '$', 50, 1, ContainerType::AREA, 130);
    let bronze_bar = Item::new_with_form(Uuid::new_v4(), "".to_owned(), MaterialType::BRONZE, ItemForm::BAR, 'X', 1, 50);
    let mut bag = container::build(Uuid::new_v4(), "Bag".to_owned(), '$', 5, 50, ContainerType::OBJECT, 50);
    let mut carton = container::build(Uuid::new_v4(), "Carton".to_owned(), '$', 1, 50, ContainerType::OBJECT, 5);
    let tin_bar = Item::new_with_form(Uuid::new_v4(), "".to_owned(), MaterialType::TIN, ItemForm::BAR,'X', 1, 50);

    // +1 weight
    carton.add_item(tin_bar);

    bag.add(carton);
    bag.add_item(bronze_bar);

    // +8 weight (bad contains 3 weight)
    container.add(bag);

    // + 60 weight
    for i in 1..=60 {
        let test_item = Item::new(Uuid::new_v4(), format!("Test Item {}", i), MaterialType::UNKNOWN, '$', 1, 100);
        container.add_item(test_item);
    }
    return container;
}


pub fn build_dev_chest() -> Container {
    let mut container = container::build(Uuid::new_v4(), "Chest".to_owned(), '$', 50, 1, ContainerType::OBJECT, 100);

    let bronze_bar = Item::new(Uuid::new_v4(), "Bronze Bar".to_owned(), MaterialType::BRONZE, 'X', 1, 50);
    let mut bag = container::build(Uuid::new_v4(), "Bag".to_owned(), '$', 5, 50, ContainerType::OBJECT, 50);
    let mut carton = container::build(Uuid::new_v4(), "Carton".to_owned(), '$', 1, 50, ContainerType::OBJECT, 5);
    let tin_bar = Item::new(Uuid::new_v4(), "Tin Bar".to_owned(), MaterialType::TIN, 'X', 1, 50);
    carton.add_item(tin_bar);
    bag.add(carton);
    bag.add_item(bronze_bar);
    container.add(bag);
    for i in 1..=60 {
        let test_item = Item::new(Uuid::new_v4(), format!("Test Item {}", i), MaterialType::UNKNOWN, '$', 1, 100);
        container.add_item(test_item);
    }
    return container;
}

fn generate_room_containers(rng: &mut Pcg64, room: Room) -> HashMap<Position, Container> {
    let mut container_map = HashMap::new();
    let inside_area = room.get_inside_area();
    let total_area = inside_area.get_total_area();
    if total_area > 1 {
        let size_x = inside_area.get_size_x();
        let size_y = inside_area.get_size_y();
        let container_count = rng.gen_range(0..=2);
        for _i in 0..container_count {
            let random_x: u16 = rng.gen_range(0..size_x) as u16;
            let random_y: u16 = rng.gen_range(0..size_y) as u16;
            let container_position = Position { x: inside_area.start_position.x.clone() + random_x, y: inside_area.start_position.y.clone() + random_y };
            container_map.insert(container_position, build_dev_chest());
        }
    }

    container_map
}

impl <'rng> Future for MapGenerator<'rng> {
    type Output = Map;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        return if self.progress.is_done() {
            Poll::Ready(self.map.clone())
        } else {
            Poll::Pending
        }
    }
}

impl <'rng> MapGenerator<'rng> {

    // Need to run after tile additions
    pub fn add_containers(&mut self) {
        let mut area_container_count = 0;
        let area_containers = self.build_area_containers();
        for pos_container in area_containers {
            let pos = pos_container.0.clone();
            let container = pos_container.1.clone();
            self.map.containers.insert( pos, container);
            area_container_count += 1;
        }
        log::info!("Added {} general area containers to the map.", area_container_count);

        let mut room_container_count = 0;

        let rooms = &mut self.map.rooms;
        for room in rooms.iter_mut() {
            let room_containers = generate_room_containers(&mut self.rng, room.clone());
            for pos_container in &room_containers {
                let mut pos = pos_container.0.clone();
                let container = pos_container.1.clone();
                let tile_type = self.map.tiles.get_tile(pos).unwrap().tile_type;
                if tile_type == Tile::Room {
                    if let Some(c) = self.map.containers.get_mut(&mut pos) {
                        c.add(container);
                        room_container_count += 1;
                    }
                }
            }
            log::info!("Added {} containers into a room.", &room_containers.len());
        }
        log::info!("Added {} containers to rooms.", room_container_count);
    }

    fn set_step(&mut self, current_step: u16, name: String) {
        self.progress.current_step = current_step;
        self.progress.step_name = String::from(name.clone());
        log::info!("{}", name.to_string());
    }

    fn send_progress(&mut self, tx: &Sender<StepProgress>) {
        tx.send(self.progress.clone());
    }

    pub async fn generate(&mut self, tx: Sender<StepProgress>) -> Map {
        self.set_step(1, String::from("Generating map..."));
        self.send_progress(&tx);
        self.build_map();

        self.set_step(2, String::from("Adding entry/exit..."));
        self.send_progress(&tx);
        self.add_entry_and_exit();

        self.set_step(3, String::from("Applying rooms..."));
        self.send_progress(&tx);
        self.add_rooms_to_map();

        self.set_step(4, String::from("Generating containers..."));
        self.send_progress(&tx);
        self.add_containers();

        self.set_step(5, String::from("Pathfinding..."));
        self.send_progress(&tx);
        self.path_rooms();

        self.set_step(6, String::from("DONE! [ any key to start ]"));
        self.send_progress(&tx);
        return self.map.clone();
    }

    fn add_entry_and_exit(&mut self) {
        let rooms = self.map.rooms.clone();
        let rooms_len = rooms.len();

        let mut updated_entry_room = None;
        let mut entry_idx : usize = 0;
        while updated_entry_room.is_none() {
            entry_idx = self.rng.gen_range(0..rooms_len) as usize;
            let entry_room = rooms.get(entry_idx).unwrap().clone();
            log::info!("Adding entry..");
            updated_entry_room = self.add_entry(entry_room);
        }
        log::info!("Entry at pos: {:?}..", updated_entry_room.as_ref().unwrap().get_entry().unwrap());


        let mut updated_exit_room = None;
        let mut exit_idx : usize = 0;
        while updated_exit_room.is_none() {
            exit_idx = self.rng.gen_range(0..rooms_len) as usize;
            let exit_room = rooms.get(exit_idx).unwrap().clone();
            log::info!("Adding exit..");
            updated_exit_room = self.add_exit(exit_room);
        }
        log::info!("Exit at pos: {:?}..", updated_exit_room.as_ref().unwrap().get_exit().unwrap());

        let mut_rooms = &mut self.map.rooms;
        *mut_rooms.get_mut(entry_idx).unwrap() = updated_entry_room.unwrap();
        *mut_rooms.get_mut(exit_idx).unwrap() = updated_exit_room.unwrap();
    }

    fn path_rooms(&mut self){
        log::info!("Pathing rooms...");
        let tile_library = crate::map::tile::build_library();
        let corridor_tile = &tile_library[&Tile::Corridor].clone();
        let rooms = self.map.get_rooms().clone();
        for i in 0..rooms.len()-1 {
            let room1 = rooms[i].clone();
            let room2 = rooms[i+1].clone();

            for door1 in room1.get_doors() {
                let door1_position = door1.position;
                if room2.get_doors().len() > 0 {
                    let door2_position = room2.get_doors()[0].position;
                    //log::info!("Pathing from door at: {:?} to door at: {:?}", door1_position, door2_position);

                    let mut pathfinding = Pathfinding::build(door1_position);
                    let path = pathfinding.a_star_search(&self.map, door2_position);
                    if path.is_empty() {
                        log::error!("Failed to build a path..")
                    } else {
                        for position in path {
                            let tile_type = self.map.tiles.get_tile(position).unwrap().tile_type;
                            if tile_type == Tile::NoTile {
                                log::debug!("Adding corridor tile at: {:?}", position);
                                self.map.tiles.set_tile(position, corridor_tile.clone());
                            } else {
                                log::debug!("Can't add corridor here. Tile is: {:?}", tile_type);
                            }
                        }
                    }
                }
            }
        }
    }

    fn add_entry(&self, room: Room) -> Option<Room> {
        let mut updated_room = room.clone();
        let inside_area = updated_room.get_inside_area();

        let mut target = None;
        for x in inside_area.start_position.x ..= inside_area.end_position.x {
            for y in inside_area.start_position.y ..= inside_area.end_position.y {
                let possible_target = Position { x, y };
                if let Some(ex) = room.get_exit() {
                    if possible_target != ex {
                        log::info!("Potential Entry point is already an Exit..");
                        continue;
                    }
                }

                let container_type =  self.map.containers.get(&possible_target).map(|c| c.container_type.clone());
                match container_type {
                    // Area containers (Floor) should be fine
                    _AREA => {},
                    _ => {
                        log::info!("Potential Exit point has a container in the way..");
                        continue;
                    }
                }

                target = Some(possible_target);
            }
        }

        if let Some(t) = target {
            updated_room.set_entry(Some(t));
            return Some(updated_room);
        } else {
            return None;
        }
    }

    fn add_exit(&self, room: Room) -> Option<Room> {
        let mut updated_room = room.clone();
        let inside_area = updated_room.get_inside_area();

        let mut target = None;
        for x in inside_area.start_position.x .. inside_area.end_position.x {
            for y in inside_area.start_position.y .. inside_area.end_position.y {
                let possible_target = Position { x, y };
                if let Some(ex) = room.get_exit() {
                    if possible_target != ex {
                        log::info!("Potential Exit point is already an Entry..");
                        continue;
                    }
                }

                let container_type =  self.map.containers.get(&possible_target).map(|c| c.container_type.clone());
                match container_type {
                    // Area containers (Floor) should be fine
                    _area => {},
                    _ => {
                        log::info!("Potential Exit point has a container in the way..");
                        continue;
                    }
                }

                target = Some(possible_target);
            }
        }

        if let Some(t) = target {
            updated_room.set_exit(Some(t));
            return Some(updated_room);
        } else {
            return None;
        }
    }

    fn generate_room(&mut self, room_pos : Position, size: u16) -> Room {
        let room_area = build_square_area( room_pos, size);
        let mut room = build_room(room_area, Vec::new());
        let mut chosen_sides = Vec::<Side>::new();
        let room_sides = room.get_sides();

        let mut doors = Vec::new();

        let door_count = self.rng.gen_range(1..=self.max_door_count);
        let map_area = self.map_area.clone();
        let map_sides = map_area.get_sides();

        for _x in 0..door_count {
            let side : Side =  self.rng.sample(Standard);
            if !chosen_sides.contains(&side) {
                chosen_sides.push(side);

                let side_idx = room_sides.iter().position(|area_side| area_side.side == side);
                let area_side = room_sides.get(side_idx.unwrap() as usize).unwrap();
                let door_position = area_side.get_mid_point();

                // Don't allow doors at the edges of the map
                let mut valid_pos = true;
                for map_side in &map_sides {
                    if map_side.area.contains_position(door_position) {
                        valid_pos = false;
                    }
                }

                if valid_pos {
                    let door = build_door(door_position);
                    doors.push(door);
                }
            }
        }
        room.set_doors(doors);
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
        self.find_possible_room_positions();
        while total_area_usage_percentage < self.room_area_quota_percentage && self.possible_room_positions.len() > 0 {
            let random_pos = self.rng.gen_range(0..self.possible_room_positions.len());
            let position = *self.possible_room_positions.get(random_pos).unwrap();

            // Try each position 2 times with a different size
            for _x in 0..=1 {
                let size = self.rng.gen_range(self.min_room_size..=self.max_room_size);
                let potential_area = build_square_area(position, size);
                let mut position_taken = false;
                for r in &rooms {
                    let taken_area = r.get_area();
                    if taken_area.intersects_or_touches(potential_area.clone()) {
                        log::debug!("Cannot fit room area, intersection of proposed Start:{},{}, End:{},{} itersects: {},{}..{},{}",
                            potential_area.start_position.x, potential_area.start_position.y, potential_area.end_position.x,potential_area.end_position.y,
                            taken_area.start_position.x, taken_area.start_position.y, taken_area.end_position.x,taken_area.end_position.y);
                        position_taken = true;
                    }
                }


                if self.map_area.can_fit(position, size) && !position_taken && room_area_total < total_area {
                    let room = self.generate_room(position, size);
                    let room_positions = room.get_area().get_positions();
                    let room_area = room.get_area().get_total_area();
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
                    log::info!("{} potential room positions left", self.possible_room_positions.len());
                    total_area_usage_percentage += area_usage_percentage;
                    log::info!("Total room area usage: {}/{}", room_area_total, total_area);
                    log::info!("Total room area usage: {}%", total_area_usage_percentage);
                    break;
                } else {
                    log::info!("Cannot fit room of size {} at position: {:?}", size, position);
                    // Remove the positions regardless
                    let potential_area = build_square_area(position, size);
                    for unusable_pos in &potential_area.get_positions() {
                        self.remove_possible_position(*unusable_pos);
                    }
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
        let mut map_tiles = Vec::new();
        let mut row;
        for y in self.map_area.start_position.y..=self.map_area.end_position.y {
            row = Vec::new();
            for x in self.map_area.start_position.x..=self.map_area.end_position.x {
                log::debug!("New tile at: {}, {}", x,y);
                row.push( self.tile_library[&Tile::NoTile].clone());
            }
            map_tiles.push(row);
        }
        map_tiles
    }

    fn build_area_containers(&self) ->  HashMap<Position, Container> {
        let mut area_containers = HashMap::new();
        for y in self.map_area.start_position.y..=self.map_area.end_position.y {
            for x in self.map_area.start_position.x..=self.map_area.end_position.x {
                let position = Position { x, y };
                match self.map.tiles.get_tile(position) {
                    Some(td) => {
                        if td.tile_type != NoTile {
                            log::debug!("New AREA container at: {}, {}", x,y);
                            let area_container = container::build(Uuid::new_v4(), "Floor".to_owned(), '$', 0, 0,  ContainerType::AREA, 999999);
                            area_containers.insert(position, area_container);
                        }
                    },
                    _ => {}
                }
            }
        }
        area_containers
    }

    fn add_room_to_map(&mut self, room: &Room) {
        let tile_library = crate::map::tile::build_library();
        let room_tile = &tile_library[&Tile::Room].clone();
        let wall_tile = &tile_library[&Tile::Wall].clone();
        let entry_tile = &tile_library[&Tile::Entry].clone();
        let exit_tile = &tile_library[&Tile::Exit].clone();

        let inside_area = room.get_inside_area();
        let mut inside_positions = inside_area.get_positions().clone();
        if let Some(entry_pos) = room.get_entry() {
            self.map.tiles.set_tile(entry_pos, entry_tile.clone());
            if let Some(idx) = inside_positions.iter().position(|p| p == &entry_pos) {
                inside_positions.remove(idx);
            }
        }
        if let Some(exit_pos) = room.get_exit() {
            self.map.tiles.set_tile(exit_pos, exit_tile.clone());
            if let Some(idx) = inside_positions.iter().position(|p| p == &exit_pos) {
                inside_positions.remove(idx);
            }
        }
        for position in inside_positions {
            self.map.tiles.set_tile(position, room_tile.clone());
        }

        let sides = room.get_sides();
        for side in sides {
            let side_positions = side.area.get_positions();
            for position in side_positions {
                self.map.tiles.set_tile(position, wall_tile.clone());
            }
        }

        for door in room.get_doors().iter() {
            let position = door.position;
            self.map.tiles.set_tile(position, door.tile_details.clone());
        }
    }

    fn add_rooms_to_map(&mut self) {
        log::info!("Adding rooms...");
        for room in self.map.get_rooms().clone() {
            self.add_room_to_map(&room);
        }
    }

    pub(crate) fn build_map(&mut self) -> Map {
        let map_area = self.map_area.clone();
        log::info!("Constructing base tiles...");
        let map_tiles = self.build_empty_tiles();
        let map = crate::map::Map {
            area: map_area,
            tiles : Tiles { tiles : map_tiles},
            rooms: Vec::new(),
            containers: HashMap::new()
        };
        self.map = map;
        log::info!("Generating rooms..");
        let rooms = self.generate_rooms();
        self.map.rooms = rooms;
        return self.map.clone();
    }

}

impl Progressible for MapGenerator<'_> {
    fn get_progress(&self) -> StepProgress {
        self.progress.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use rand_seeder::Seeder;
    use crate::block_on;
    use crate::map::Map;
    use crate::map::map_generator::build_generator;
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::TileDetails;
    use crate::progress::StepProgress;

    fn build_test_map() -> Map {
        let map_size = 12;
        let map_area = build_square_area(Position {x: 0, y: 0}, map_size);
        let rng = &mut Seeder::from("test".to_string()).make_rng();
        let mut generator = build_generator(rng, map_area);

        let (tx, rx) = channel();
        block_on(generator.generate(tx))
    }

    #[test]
    fn test_build_generator() {
        // GIVEN a 12x12 map board
        let map_area = build_square_area(Position {x: 0, y: 0}, 12);
        let rng = &mut Seeder::from("test".to_string()).make_rng();
        let generator = build_generator(rng, map_area);

        assert_eq!(3, generator.min_room_size);
        assert_eq!(6, generator.max_room_size);
        assert_eq!(30, generator.room_area_quota_percentage);
        assert_eq!(4, generator.max_door_count);
        assert!(generator.tile_library.len() > 0);
        assert_eq!(map_area, generator.map_area);
        assert_eq!(0, generator.taken_positions.len());
        assert_eq!(0, generator.possible_room_positions.len());
    }

    #[test]
    fn test_generate_room() {
        let map_area = build_square_area(Position {x: 0, y: 0}, 12);
        let rng = &mut Seeder::from("test".to_string()).make_rng();
        let mut generator = build_generator(rng, map_area);

        let room = generator.generate_room(Position {x: 0, y: 0}, 3);
        let expected_area = build_square_area(Position {x: 0, y: 0}, 3);
        assert_eq!(expected_area, room.get_area());
        assert!(!room.get_doors().is_empty());
    }

    #[test]
    fn test_generate_rooms() {
        let map_size = 12;
        let map_area = build_square_area(Position {x: 0, y: 0}, map_size);
        let rng = &mut Seeder::from("test".to_string()).make_rng();
        let mut generator = build_generator(rng, map_area);
        let rooms = generator.generate_rooms();
        assert_ne!(0, rooms.len());

        for room in rooms {
            let area = room.get_area();
            let start_pos = area.start_position;
            assert!(start_pos.x <= 12 && start_pos.y < 12, "Expected room start position < 12 for x,y, but was: {}, {}", start_pos.x, start_pos.y);
            let end_pos = area.end_position;
            assert!(end_pos.x <= 12 && end_pos.y < 12, "Expected room end position < 12 for x,y, but was: {}, {}", end_pos.x, end_pos.y);
        }
    }

    fn assert_string_vecs(expected: Vec<String>, actual: Vec<String>) {
        let mut expected_full = String::from("");
        for line in &expected {
            expected_full = format!("{}\n{}", expected_full, line);
        }

        let mut actual_full = String::from("");
        for line in &actual {
            actual_full = format!("{}\n{}", actual_full, line);
        }

        assert_eq!(actual_full, expected_full);
    }

    fn build_tile_strings(length: i32, tiles: &Vec<Vec<TileDetails>>) -> Vec<String> {
        let mut tile_strings : Vec<String> = Vec::new();
        for i in 0..length {
            tile_strings.push("".to_string())
        }

        let mut x_idx = 0;
        let mut y_idx = 0;
        for row in tiles {
            let mut row_text = tile_strings.get_mut(x_idx).unwrap().clone();
            for tile in row {
                tile_strings[x_idx].push(tile.symbol.character);
                y_idx += 1;
            }
            x_idx += 1;
        }
        tile_strings
    }

    #[test]
    fn test_generate() {
        let map = build_test_map();

        let area = map.area;
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(11, area.end_position.x);
        assert_eq!(11, area.end_position.y);

        let tiles = map.tiles.tiles;
        assert_eq!(12, tiles.len());
        for row in &tiles {
            assert_eq!(12, row.len());
        }

        let rooms = map.rooms;
        assert_ne!(0, rooms.len());
        for room in rooms {
            let area = room.get_area();
            let start_pos = area.start_position;
            assert!(start_pos.x <= 12 && start_pos.y < 12, "Expected room start position < 12 for x,y, but was: {}, {}", start_pos.x, start_pos.y);
            let end_pos = area.end_position;
            assert!(end_pos.x <= 12 && end_pos.y < 12, "Expected room end position < 12 for x,y, but was: {}, {}", end_pos.x, end_pos.y);
        }

        let expected_tiles : Vec<String>  = vec![
            "            ".to_string(),
            "            ".to_string(),
            "            ".to_string(),
            " #####      ".to_string(),
            " #---#      ".to_string(),
            " #-^-=-     ".to_string(),
            " #---#-     ".to_string(),
            " #####---#=#".to_string(),
            "        -=^#".to_string(),
            "         #=#".to_string(),
            "            ".to_string(),
            "            ".to_string()
        ];

        let actual_tiles = build_tile_strings(12, &tiles);
        assert_string_vecs(expected_tiles, actual_tiles);
    }
}