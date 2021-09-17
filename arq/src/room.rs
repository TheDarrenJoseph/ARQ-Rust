use crate::position::{Area,AreaSide,all_sides, build_line};
use crate::door::Door;

pub struct Room {
    pub area: Area,
    pub doors : Vec<Door>
}

impl Room {
    pub fn get_sides(&self) -> Vec<AreaSide> {
        let start_pos = &self.area.start_position;
        let size = &self.area.size;
        let mut sides = Vec::new();
        for side in all_sides().iter() {
            sides.push(build_line(start_pos.clone(), *size, side.clone()));
        }
        sides
    }
}

#[cfg(test)]
mod tests {
    use crate::door::build_door;
    use crate::room::Room;
    use crate::position::{Position, Area, Side, build_area};

    #[test]
    fn test_get_sides() {
        let start_position = Position { x: 0, y: 0};
        let area = build_area(start_position, 3);

        let door_position = Position { x: 1, y: 0};
        let door = build_door(door_position);
        let mut doors = Vec::new();
        doors.push(door);
        let room = Room { area, doors };

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
}