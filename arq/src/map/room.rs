use rand::Rng;
use rand::rngs::ThreadRng;
use uuid::Uuid;

use crate::map::objects::door::Door;
use crate::map::position::{Area, AreaSide, build_rectangular_area, Position};
use crate::util::utils::{HasUuid, UuidEquals};

#[derive(Debug, Clone)]
pub struct Room {
    id: Uuid,
    area: Area,
    doors : Vec<Door>,
    entry: Option<Position>,
    exit : Option<Position>
}

impl HasUuid for Room {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl UuidEquals<Room> for Room {}

pub fn build_room (area: Area, doors: Vec<Door>)-> Room {
    return Room { id: Uuid::new_v4(), area, doors, entry: None, exit: None };
}

impl Room {
    pub fn get_sides(&self) -> Vec<AreaSide> {
        self.area.get_sides()
    }

    pub fn get_inside_area(&self) -> Area {
        let start_pos = &self.area.start_position;
        let start_position = Position { x : start_pos.x + 1, y: start_pos.y + 1};
        build_rectangular_area(start_position,  self.area.get_size_x()-2,  self.area.get_size_y() - 2 )
    }

    pub fn random_inside_pos(&self, rng: &mut ThreadRng) -> Position {
        let inside_area = self.get_inside_area();
        let size_x = inside_area.get_size_x();
        let size_y = inside_area.get_size_y();
        let random_x: u16 = rng.gen_range(0..size_x) as u16;
        let random_y: u16 = rng.gen_range(0..size_y) as u16;
        let random_pos = Position { x: inside_area.start_position.x.clone() + random_x, y: inside_area.start_position.y.clone() + random_y };
        return random_pos;
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_area(&self) -> Area {
        self.area
    }

    pub fn get_doors(&self) -> &Vec<Door> {
        &self.doors
    }

    pub fn set_doors(&mut self, doors: Vec<Door>) {
        self.doors = doors;
    }

    pub fn get_entry(&self) -> Option<Position> {
        self.entry
    }

    pub fn set_entry(&mut self, entry: Option<Position>) {
        self.entry = entry;
    }

    pub fn get_exit(&self) -> Option<Position> {
        self.exit
    }

    pub fn set_exit(&mut self, exit: Option<Position>) {
        self.exit = exit;
    }
}

#[cfg(test)]
mod tests {
    use crate::map::objects::door::build_door;
    use crate::map::position::{build_square_area, Position, Side};
    use crate::map::room::build_room;

    #[test]
    fn test_get_sides() {
        let start_position = Position { x: 0, y: 0};
        let area = build_square_area(start_position, 3);

        let door_position = Position { x: 1, y: 0};
        let door = build_door(door_position);
        let mut doors = Vec::new();
        doors.push(door);

        let room = build_room(area, doors);
        let sides = room.get_sides();

        assert_eq!(4, sides.len());

        let left = sides[0];
        assert_eq!(Side::LEFT, left.side);
        assert_eq!(0, left.area.start_position.x);
        assert_eq!(0, left.area.start_position.y);
        assert_eq!(0, left.area.end_position.x);
        assert_eq!(2, left.area.end_position.y);

        let right = sides[1];
        assert_eq!(Side::RIGHT, right.side);
        assert_eq!(2, right.area.start_position.x);
        assert_eq!(0, right.area.start_position.y);
        assert_eq!(2, right.area.end_position.x);
        assert_eq!(2, right.area.end_position.y);

        let top = sides[2];
        assert_eq!(Side::TOP, top.side);
        assert_eq!(0, top.area.start_position.x);
        assert_eq!(0, top.area.start_position.y);
        assert_eq!(2, top.area.end_position.x);
        assert_eq!(0, top.area.end_position.y);

        let bottom = sides[3];
        assert_eq!(Side::BOTTOM, bottom.side);
        assert_eq!(0, bottom.area.start_position.x);
        assert_eq!(2, bottom.area.start_position.y);
        assert_eq!(2, bottom.area.end_position.x);
        assert_eq!(2, bottom.area.end_position.y);
    }

    #[test]
    fn test_get_inside_area() {
        // GIVEN a room of 4x4
        let start_position = Position { x: 0, y: 0};
        let area = build_square_area(start_position, 4);
        let room = build_room(area, Vec::new());
        assert_eq!(Position { x: 0, y: 0}, room.area.start_position);
        assert_eq!(Position { x: 3, y: 3}, room.area.end_position);

        // WHEN we call to get the inside area
        let inside_area = room.get_inside_area();

        // THEN we expect it to start at 1,1
        assert_eq!(Position { x: 1, y: 1}, inside_area.start_position);
        // AND end at 2,2
        assert_eq!(Position { x: 2, y: 2}, inside_area.end_position);

        assert_eq!(4, inside_area.get_total_area());
    }

    #[test]
    fn test_get_inside_area_realworld() {
        // GIVEN a room of 4x4
        let start_position = Position { x: 2, y: 6};
        let area = build_square_area(start_position, 6);
        let room = build_room(area, Vec::new());
        assert_eq!(Position { x: 2, y: 6}, room.area.start_position);
        assert_eq!(Position { x: 7, y: 11}, room.area.end_position);

        // WHEN we call to get the inside area
        let inside_area = room.get_inside_area();

        // THEN we expect it to start at 1,1
        assert_eq!(Position { x: 3, y: 7}, inside_area.start_position);
        // AND end at 2,2
        assert_eq!(Position { x: 6, y: 10}, inside_area.end_position);

        assert_eq!(16, inside_area.get_total_area());
    }
}