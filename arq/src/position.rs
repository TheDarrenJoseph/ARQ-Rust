use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Copy, Clone, std::cmp::PartialEq, Debug)]
pub struct Position {
    pub x : u16,
    pub y : u16
}

#[derive(Copy, Clone, std::cmp::PartialEq, Debug)]
pub struct Area {
    pub start_position : Position,
    pub end_position : Position,
    size_x : u16,
    size_y : u16
}

impl Area {
    pub fn get_total_area(&self) -> u16 {
        self.size_x * self.size_y
    }

    pub fn get_size_x(&self) -> u16 {
        self.size_x
    }

    pub fn get_size_y(&self) -> u16 {
        self.size_y
    }

    pub fn get_sides(&self) -> Vec<AreaSide> {
        let start_pos = &self.start_position;
        let mut sides = Vec::new();
        for side in all_sides().iter() {
            if *side == Side::LEFT || *side == Side::RIGHT {
                sides.push(build_line(start_pos.clone(), self.size_y, side.clone()));
            } else {
                sides.push(build_line(start_pos.clone(), self.size_x, side.clone()));
            }
        }
        sides
    }

    pub fn intersects(&self, area: Area) -> bool {

        let start_x = self.start_position.x;
        let start_y = self.start_position.y;
        let end_x = self.end_position.x;
        let end_y = self.end_position.y;

        let area_start_x = area.start_position.x;
        let area_start_y = area.start_position.y;

        let area_end_x = area.end_position.x;
        let area_end_y = area.end_position.y;

        let area_start_x_intersect = area_start_x >= start_x && area_start_x <= end_x;
        let area_end_x_intersect = area_end_x >= start_x && area_end_x <= end_x;

        let area_start_y_intersect = area_start_y >= start_y && area_start_y <= end_y;
        let area_end_y_intersect = area_end_y >= start_y && area_end_y <= end_y;

        let intersects = (area_start_x_intersect || area_end_x_intersect) && (area_start_y_intersect || area_end_y_intersect);
        intersects
    }


    pub fn intersects_or_touches(&self, area: Area) -> bool {
        // Adjust boundaries to allow 1 place above or below each range
        let start_x = if self.start_position.x > 0 { self.start_position.x - 1 } else { self.start_position.x };
        let start_y = if self.start_position.y > 0 {  self.start_position.y - 1 } else {  self.start_position.y };
        let end_x = if self.end_position.x < u16::MAX { self.end_position.x + 1 } else { self.end_position.x };
        let end_y = if self.end_position.y < u16::MAX { self.end_position.y + 1 } else { self.end_position.y };

        // Cast to signed and get absolute value to find the diff
        let size_x = (start_x as i32 - end_x as i32).abs() as u16;
        let size_y = (start_y as i32 - end_y as i32).abs() as u16;

        let start_position = Position { x: start_x, y: start_y };
        let end_position = Position { x: end_x, y: end_y};

        let self_adjusted_area = Area { start_position, end_position, size_x, size_y };
        self_adjusted_area.intersects(area)
    }

    pub fn contains(&self, position: Position) -> bool {
        let x= position.x;
        let y = position.y;

        let lower_x_bound = x >= self.start_position.x;
        let lower_y_bound = y >= self.start_position.y;
        let upper_x_bound = x <= self.end_position.x;
        let upper_y_bound = y <= self.end_position.y;
        let in_bounds = lower_x_bound && lower_y_bound && upper_x_bound && upper_y_bound;
        in_bounds
    }

    pub fn can_fit(&self, position: Position, size: u16) -> bool {
        if size == 0 {
            return false;
        }

        let x= position.x;
        let y = position.y;
        let in_bounds = self.contains(position);
        if in_bounds && size <= self.size_x && size <= self.size_y {
            let end_x = x + size - 1;
            let end_y = y + size - 1;

            let end_x_bound = end_x <= self.end_position.x;
            let end_y_bound = end_y <= self.end_position.y;
            let end_bound_fit = end_x_bound && end_y_bound;
            return end_bound_fit;
        }
        false
    }

    pub fn get_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        let start_pos = self.start_position;
        let end_pos = self.end_position;
        for x in start_pos.x..end_pos.x+1  {
            for y in start_pos.y..end_pos.y+1 {
                positions.push(Position { x, y });
            }
        }
        positions
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Side {
    LEFT,
    RIGHT,
    TOP,
    BOTTOM
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0..8) {
            0 => Side::LEFT,
            1 => Side::RIGHT,
            2 => Side::TOP,
            3 => Side::BOTTOM,
            _ => Side::LEFT,
        }
    }
}

#[derive(Copy, Clone)]
pub struct AreaSide {
    pub area: Area,
    pub side : Side
}

impl AreaSide {
    pub fn get_mid_point(&self) -> Position {
       match self.side {
           Side::LEFT => {
               Position { x: self.area.start_position.x, y: self.area.start_position.y + (self.area.size_y - 1) / 2 }
           },
           Side::RIGHT => {
               Position { x: self.area.end_position.x, y: self.area.start_position.y + (self.area.size_y - 1) / 2 }
           },
           Side::TOP => {
               Position { x: self.area.start_position.x + (self.area.size_x - 1) / 2, y: self.area.start_position.y}
           },
           Side::BOTTOM => {
               Position { x: self.area.start_position.x + (self.area.size_x - 1) / 2, y: self.area.end_position.y }
           }
       }
    }
}

pub fn all_sides() -> [Side; 4] {
    [Side::LEFT,Side::RIGHT,Side::TOP,Side::BOTTOM]
}

pub fn build_line(start_position : Position, size: u16, side: Side) -> AreaSide {
    let start_x = start_position.x;
    let start_y = start_position.y;

    let start_position;
    let end_position;
    let zero_indexed_size = size - 1;
    let size_x;
    let size_y;
    match side {
        Side::LEFT => {
            start_position = Position { x : start_x, y: start_y};
            end_position = Position { x : start_x, y: start_y + zero_indexed_size};
            size_x = 1;
            size_y = size;
        },
        Side::RIGHT => {
            start_position = Position { x : start_x + zero_indexed_size, y: start_y};
            end_position = Position { x : start_x + zero_indexed_size, y: start_y + zero_indexed_size};
            size_x = 1;
            size_y = size;
        },
        Side::TOP => {
            start_position = Position { x : start_x, y: start_y};
            end_position = Position { x : start_x + zero_indexed_size, y: start_y};
            size_x = size;
            size_y = 1;
        },
        Side::BOTTOM => {
            start_position = Position { x : start_x, y: start_y + zero_indexed_size};
            end_position = Position { x : start_x + zero_indexed_size, y: start_y + zero_indexed_size};
            size_x = size;
            size_y = 1;
        }
    }
    let area = Area { start_position, end_position, size_x, size_y };
    AreaSide { area, side }
}

pub fn build_square_area(start_position : Position, size: u16) -> Area {
    let start_x = start_position.x;
    let start_y = start_position.y;
    let end_position = Position { x : start_x + (size - 1), y: start_y + (size - 1)};
    Area { start_position, end_position, size_x: size, size_y: size }
}

pub fn build_rectangular_area(start_position : Position, size_x: u16, size_y: u16) -> Area {
    let start_x = start_position.x;
    let start_y = start_position.y;
    let end_position = Position { x : start_x + (size_x - 1), y: start_y + (size_y - 1)};
    Area { start_position, end_position, size_x, size_y }
}

#[cfg(test)]
mod tests {
    use crate::position::{Position, Area, Side, AreaSide, build_square_area, build_rectangular_area};

    #[test]
    fn test_build_square_area() {
        let start_pos = Position { x: 1, y: 2 };
        let area = build_square_area(start_pos, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(2, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(4, area.end_position.y);
    }

    #[test]
    fn test_build_rectangular_area() {
        let start_pos = Position { x: 0, y: 0 };
        let area = build_rectangular_area(start_pos, 6, 3);
        assert_eq!(6, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(5, area.end_position.x);
        assert_eq!(2, area.end_position.y);
    }

    #[test]
    fn test_get_total_area() {
        // GIVEN a valid area
        let start_pos = Position { x: 0, y: 0 };
        let area = build_square_area(start_pos, 4);
        assert_eq!(4, area.size_x);
        assert_eq!(4, area.size_y);
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to get the total area
        let total_area = area.get_total_area();

        // THEN we expect it to equal width (x) * height (y) / 4x4
        assert_eq!(16, total_area);
    }

    #[test]
    fn test_get_get_positions() {
        // GIVEN a valid area
        let start_pos = Position { x: 0, y: 0 };
        let area = build_square_area(start_pos, 4);
        assert_eq!(4, area.size_x);
        assert_eq!(4, area.size_y);
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to get all possible positions
        let positions = area.get_positions();

        // THEN we expect it to equal width (x) * height (y) / 4x4
        assert_eq!(16, positions.len());
       // First column
        assert_eq!(Position { x: 0, y: 0}, *positions.get(0).unwrap());
        assert_eq!(Position { x: 0, y: 1}, *positions.get(1).unwrap());
        assert_eq!(Position { x: 0, y: 2}, *positions.get(2).unwrap());
        assert_eq!(Position { x: 0, y: 3}, *positions.get(3).unwrap());

        // Second column
        assert_eq!(Position { x: 1, y: 0}, *positions.get(4).unwrap());
        assert_eq!(Position { x: 1, y: 1}, *positions.get(5).unwrap());
        assert_eq!(Position { x: 1, y: 2}, *positions.get(6).unwrap());
        assert_eq!(Position { x: 1, y: 3}, *positions.get(7).unwrap());

        // Third column
        assert_eq!(Position { x: 2, y: 0}, *positions.get(8).unwrap());
        assert_eq!(Position { x: 2, y: 1}, *positions.get(9).unwrap());
        assert_eq!(Position { x: 2, y: 2}, *positions.get(10).unwrap());
        assert_eq!(Position { x: 2, y: 3}, *positions.get(11).unwrap());

        // Fourth column
        assert_eq!(Position { x: 3, y: 0}, *positions.get(12).unwrap());
        assert_eq!(Position { x: 3, y: 1}, *positions.get(13).unwrap());
        assert_eq!(Position { x: 3, y: 2}, *positions.get(14).unwrap());
        assert_eq!(Position { x: 3, y: 3}, *positions.get(15).unwrap());
    }

    #[test]
    fn test_get_get_positions_nonzero_start() {
        // GIVEN a valid area starting at index 4, 4
        let start_pos = Position { x: 4, y: 4 };
        let area = build_square_area(start_pos, 4);
        assert_eq!(4, area.size_x);
        assert_eq!(4, area.size_y);
        assert_eq!(4, area.start_position.x);
        assert_eq!(4, area.start_position.y);
        assert_eq!(7, area.end_position.x);
        assert_eq!(7, area.end_position.y);

        // WHEN we call to get all possible positions
        let positions = area.get_positions();

        // THEN we expect it to equal width (x) * height (y) / 4x4
        assert_eq!(16, positions.len());
        // First column
        assert_eq!(Position { x: 4, y: 4}, *positions.get(0).unwrap());
        assert_eq!(Position { x: 4, y: 5}, *positions.get(1).unwrap());
        assert_eq!(Position { x: 4, y: 6}, *positions.get(2).unwrap());
        assert_eq!(Position { x: 4, y: 7}, *positions.get(3).unwrap());

        // Second column
        assert_eq!(Position { x: 5, y: 4}, *positions.get(4).unwrap());
        assert_eq!(Position { x: 5, y: 5}, *positions.get(5).unwrap());
        assert_eq!(Position { x: 5, y: 6}, *positions.get(6).unwrap());
        assert_eq!(Position { x: 5, y: 7}, *positions.get(7).unwrap());

        // Third column
        assert_eq!(Position { x: 6, y: 4}, *positions.get(8).unwrap());
        assert_eq!(Position { x: 6, y: 5}, *positions.get(9).unwrap());
        assert_eq!(Position { x: 6, y: 6}, *positions.get(10).unwrap());
        assert_eq!(Position { x: 6, y: 7}, *positions.get(11).unwrap());

        // Fourth column
        assert_eq!(Position { x: 7, y: 4}, *positions.get(12).unwrap());
        assert_eq!(Position { x: 7, y: 5}, *positions.get(13).unwrap());
        assert_eq!(Position { x: 7, y: 6}, *positions.get(14).unwrap());
        assert_eq!(Position { x: 7, y: 7}, *positions.get(15).unwrap());
    }

    #[test]
    fn test_3x3_valid_top_left_intersects() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 1, y: 1 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(1, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to see if an area overlaping the top-left corner intersects
        let overlap_area = build_square_area(Position { x: 0, y: 0 }, 2);
        // THEN we expect the result to be true
        assert!(area.intersects(overlap_area));
    }

    #[test]
    fn test_3x3_valid_top_right_intersects() {
        // GIVEN a 3x3 Area (x)
        let area = build_square_area(Position { x: 1, y: 1 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(1, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to see if an area (X) overlaping the top-right corner intersects
        // i.e
        // ---XX
        // -xxXX
        // -xxx-
        // -xxx-
        let overlap_area = build_square_area(Position { x: 3, y: 0 }, 2);
        // THEN we expect the result to be true
        assert!(area.intersects(overlap_area));
    }

    #[test]
    fn test_3x3_valid_bottom_right_intersects() {
        // GIVEN a 3x3 Area (x)
        let area = build_square_area(Position { x: 1, y: 1 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(1, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to see if an area (X) overlaping the top-right corner intersects
        // i.e
        //  01234
        // 0-----
        // 1-xxx-
        // 2-xxx-
        // 3-xxXX
        // 4---XX
        let overlap_area = build_square_area(Position { x: 3, y: 3 }, 2);
        // THEN we expect the result to be true
        assert!(area.intersects(overlap_area));
    }

    #[test]
    fn test_3x3_valid_bottom_left_intersects() {
        // GIVEN a 3x3 Area (x)
        let area = build_square_area(Position { x: 1, y: 1 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(1, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to see if an area (X) overlaping the top-right corner intersects
        // i.e
        //  01234
        // 0-----
        // 1-xxx-
        // 2-xxx-
        // 3XXxx-
        // 4XX---
        let overlap_area = build_square_area(Position { x: 0, y: 3 }, 2);
        // THEN we expect the result to be true
        assert!(area.intersects(overlap_area));
    }

    #[test]
    fn test_3x3_invalid_ends_before_intersects() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 1, y: 1 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(1, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to see if an area that ends touching the top-left corner intersects
        let overlap_area = build_square_area(Position { x: 0, y: 0 }, 1);
        // THEN we expect the result to be false
        assert!(!area.intersects(overlap_area));
    }

    #[test]
    fn test_3x3_invalid_starts_after_intersects() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 1, y: 1 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(1, area.start_position.x);
        assert_eq!(1, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(3, area.end_position.y);

        // WHEN we call to see if an area that starts after the bottom-right corner intersects
        let overlap_area = build_square_area(Position { x: 4, y: 4 }, 3);
        // THEN we expect the result to be false
        assert!(!area.intersects(overlap_area));
    }

    #[test]
    fn test_3x3_valid_contains() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(2, area.end_position.x);
        assert_eq!(2, area.end_position.y);

        // WHEN we call to see if points 0,0 - 2,2 are contained
        // THEN we expect the results to be true
        assert!(area.contains(Position { x: 0, y: 0 }));
        assert!(area.contains(Position { x: 0, y: 1 }));
        assert!(area.contains(Position { x: 0, y: 2 }));
        assert!(area.contains(Position { x: 1, y: 0 }));
        assert!(area.contains(Position { x: 1, y: 1 }));
        assert!(area.contains(Position { x: 1, y: 2 }));
        assert!(area.contains(Position { x: 2, y: 0 }));
        assert!(area.contains(Position { x: 2, y: 1 }));
        assert!(area.contains(Position { x: 2, y: 2 }));
    }

    #[test]
    fn test_3x3_invalid_contains() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);
        assert_eq!(3, area.size_x);
        assert_eq!(3, area.size_y);
        assert_eq!(0, area.start_position.x);
        assert_eq!(0, area.start_position.y);
        assert_eq!(2, area.end_position.x);
        assert_eq!(2, area.end_position.y);

        // WHEN we call to see if points above it's range are contained
        // THEN we expect the results to be false
        assert!(!area.contains(Position { x: 3, y: 0 }));
        assert!(!area.contains(Position { x: 0, y: 3 }));
        assert!(!area.contains(Position { x: 3, y: 3 }));
    }

    #[test]
    fn test_3x3_can_fit_2x2_at_1_1() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);

        // WHEN we call to see if the central point can fit a 2x2 area
        let can_fit = area.can_fit(Position { x: 1, y: 1 }, 2);

        // THEN we expect the result to be true
        assert_eq!(true, can_fit);
    }

    #[test]
    fn test_3x3_can_fit_2x2_at_start() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);

        // WHEN we call to see if the start point can fit a 2x2 area
        let can_fit = area.can_fit(Position { x: 0, y: 0 }, 2);

        // THEN we expect the result to be true
        assert_eq!(true, can_fit);
    }

    #[test]
    fn test_3x3_cannot_fit_2x2_at_end() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);

        // WHEN we call to see if the end point can fit a 2x2 area
        let can_fit = area.can_fit(Position { x: 2, y: 2 }, 4);

        // THEN we expect the result to be false
        assert_ne!(true, can_fit);
    }

    #[test]
    fn test_3x3_cannot_fit_4x4_at_start() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);

        // WHEN we call to see if the start point can fit a 4x4 area
        let can_fit = area.can_fit(Position { x: 0, y: 0 }, 4);

        // THEN we expect the result to be false
        assert_ne!(true, can_fit);
    }

    #[test]
    fn test_3x3_cannot_fit_zero() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);

        // WHEN we call to see if the start point can fit a zero sized area
        let can_fit = area.can_fit(Position { x: 0, y: 0 }, 0);

        // THEN we expect the result to be false
        assert_ne!(true, can_fit);
    }

    #[test]
    fn test_get_sides() {
        let start_position = Position { x: 0, y: 0};
        let area = build_square_area(start_position, 3);
        let sides = area.get_sides();

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