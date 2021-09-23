#[derive(Copy, Clone, std::cmp::PartialEq, Debug)]
pub struct Position {
    pub x : u16,
    pub y : u16
}

#[derive(Copy, Clone, std::cmp::PartialEq, Debug)]
pub struct Area {
    pub start_position : Position,
    pub end_position : Position,
    pub size : u16
}

impl Area {
    pub fn get_total_area(&self) -> u16 {
        self.size * self.size
    }

    pub fn intersects(&self, area: Area) -> bool {
        for position in area.get_positions() {
            if self.contains(position) {
                return true;
            }
        }
        return false;
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
        if in_bounds && size <= self.size {
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

#[derive(Copy, Clone)]
pub struct AreaSide {
    pub area: Area,
    pub side : Side
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
    match side {
        Side::LEFT => {
            start_position = Position { x : start_x, y: start_y};
            end_position = Position { x : start_x, y: start_y + zero_indexed_size};
        },
        Side::RIGHT => {
            start_position = Position { x : start_x + zero_indexed_size, y: start_y};
            end_position = Position { x : start_x + zero_indexed_size, y: start_y + zero_indexed_size};
        },
        Side::TOP => {
            start_position = Position { x : start_x, y: start_y};
            end_position = Position { x : start_x + zero_indexed_size, y: start_y};
        },
        Side::BOTTOM => {
            start_position = Position { x : start_x, y: start_y + zero_indexed_size};
            end_position = Position { x : start_x + zero_indexed_size, y: start_y + zero_indexed_size};
        }
    }
    let area = Area { start_position, end_position, size };
    AreaSide { area, side }
}

pub fn build_square_area(start_position : Position, size: u16) -> Area {
    let start_x = start_position.x;
    let start_y = start_position.y;
    let end_position = Position { x : start_x + (size - 1), y: start_y + (size - 1)};
    Area { start_position, end_position, size }
}

#[cfg(test)]
mod tests {
    use crate::position::{Position, Area, build_square_area};

    #[test]
    fn test_build_square_area() {
        let start_pos = Position { x: 1, y: 2 };
        let area = build_square_area(start_pos, 3);
        assert_eq!(3, area.size);
        assert_eq!(1, area.start_position.x);
        assert_eq!(2, area.start_position.y);
        assert_eq!(3, area.end_position.x);
        assert_eq!(4, area.end_position.y);
    }

    #[test]
    fn test_get_total_area() {
        // GIVEN a valid area
        let start_pos = Position { x: 0, y: 0 };
        let area = build_square_area(start_pos, 4);
        assert_eq!(4, area.size);
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
        assert_eq!(4, area.size);
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
        assert_eq!(4, area.size);
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
    fn test_3x3_valid_contains() {
        // GIVEN a 3x3 Area
        let area = build_square_area(Position { x: 0, y: 0 }, 3);
        assert_eq!(3, area.size);
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
        assert_eq!(3, area.size);
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
}