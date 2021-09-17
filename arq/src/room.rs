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
