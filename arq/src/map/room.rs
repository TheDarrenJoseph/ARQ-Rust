use crate::map::objects::door::Door;
use crate::map::position::{Area, AreaSide, build_rectangular_area, Position};

#[derive(Clone)]
pub struct Room {
    pub area: Area,
    pub doors : Vec<Door>,
    pub entry: Option<Position>,
    pub exit : Option<Position>
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::map::objects::door::build_door;
    use crate::map::position::{build_square_area, Position, Side};
    use crate::map::room::Room;

    #[test]
    fn test_get_sides() {
        let start_position = Position { x: 0, y: 0};
        let area = build_square_area(start_position, 3);

        let door_position = Position { x: 1, y: 0};
        let door = build_door(door_position);
        let mut doors = Vec::new();
        doors.push(door);
        let room = Room { area, doors, entry: None, exit: None };

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
        let doors = Vec::new();
        let room = Room { area, doors, entry: None, exit: None };
        assert_eq!(Position { x: 0, y: 0}, room.area.start_position);
        assert_eq!(Position { x: 3, y: 3}, room.area.end_position);

        // WHEN we call to get the inside area
        let inside_area = room.get_inside_area();

        // THEN we expect it to start at 1,1
        assert_eq!(Position { x: 1, y: 1}, inside_area.start_position);
        // AND end at 2,2
        assert_eq!(Position { x: 2, y: 2}, inside_area.end_position);
    }
}