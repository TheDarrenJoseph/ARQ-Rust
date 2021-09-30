use crate::position::Position;

trait ManhattanPathCosting {
    fn manhattan_path_cost(&self, a: Position, b: Position) -> i32;
}

struct Pathfinding {

}

impl ManhattanPathCosting for Pathfinding {
    fn manhattan_path_cost(&self, a: Position, b: Position) -> i32 {
        let x_abs = (a.x as i32 - b.x as i32).abs() as i32;
        let y_abs = (a.y as i32 - b.y as i32).abs() as i32;
        x_abs + y_abs
    }
}

#[cfg(test)]
mod tests {
    use crate::pathfinding;
    use crate::position::Position;
    use crate::pathfinding::{ManhattanPathCosting, Pathfinding};

    #[test]
    fn test_manhattan_path_cost_zero() {
        // GIVEN 2 positions that are equivalent
        let start_pos = Position { x: 1, y: 3 };
        let end_pos = start_pos.clone();

        // WHEN we call to get the manhattan cost for the distance between these 2 points
        let pathfinding = Pathfinding{};
        let result = pathfinding.manhattan_path_cost(start_pos, end_pos);

        // THEN we expect 0 to be returned
        assert_eq!(0, result);
    }

    #[test]
    fn test_manhattan_path_cost_nonzero_xdiff() {
        // GIVEN 2 positions
        let start_pos = Position { x: 1, y: 3 };
        // And the 2nd differs only in the x coord by 2
        let end_pos =  Position { x: 3, y: 3 };

        // WHEN we call to get the manhattan cost for the distance between these 2 points
        let pathfinding = Pathfinding{};
        let result = pathfinding.manhattan_path_cost(start_pos, end_pos);

        // THEN we expect 2 to be returned
        assert_eq!(2, result);
    }

    #[test]
    fn test_manhattan_path_cost_nonzero_ydiff() {
        // GIVEN 2 positions
        let start_pos = Position { x: 1, y: 3 };
        // And the 2nd differs only in the y coord by 2
        let end_pos =  Position { x: 1, y: 5 };

        // WHEN we call to get the manhattan cost for the distance between these 2 points
        let pathfinding = Pathfinding{};
        let result = pathfinding.manhattan_path_cost(start_pos, end_pos);

        // THEN we expect 2 to be returned
        assert_eq!(2, result);
    }

    #[test]
    fn test_manhattan_path_cost_nonzero() {
        // GIVEN 2 positions
        let start_pos = Position { x: 1, y: 3 };
        // And the 2nd differs by 2
        let end_pos =  Position { x: 3, y: 5 };

        // WHEN we call to get the manhattan cost for the distance between these 2 points
        let pathfinding = Pathfinding{};
        let result = pathfinding.manhattan_path_cost(start_pos, end_pos);

        // THEN we expect 4 to be returned
        assert_eq!(4, result);
    }
}