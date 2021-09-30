use std::collections::{BinaryHeap,VecDeque,HashMap};
use std::cmp::{Reverse,Ordering};

use crate::map::Map;
use crate::position::Position;

trait ManhattanPathCosting {
    fn manhattan_path_cost(&self, a: Position, b: Position) -> i32;
}

#[derive(Clone, Hash, Debug)]
struct Node {
    position: Position,
    score: i32
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Node {}

struct Pathfinding {
    start_position: Position,
    unvisited : BinaryHeap<Reverse<Node>>,
    previous_nodes: VecDeque<Node>,
    g_scores:   HashMap<Position, i32>,
    f_scores:   HashMap<Position, i32>,
}

impl ManhattanPathCosting for Pathfinding {
    fn manhattan_path_cost(&self, a: Position, b: Position) -> i32 {
        let x_abs = (a.x as i32 - b.x as i32).abs() as i32;
        let y_abs = (a.y as i32 - b.y as i32).abs() as i32;
        x_abs + y_abs
    }
}

impl Pathfinding {
    pub fn build(start_position: Position) -> Pathfinding {
        let mut pathfinding = Pathfinding { start_position, unvisited: BinaryHeap::new(), previous_nodes: VecDeque::new(), g_scores: HashMap::new(), f_scores: HashMap::new() };
        let node = Node { position: start_position, score: 0 };
        let reversed = Reverse(node);
        pathfinding.unvisited.push(reversed);
        pathfinding.g_scores.insert(start_position, 0);
        pathfinding
    }

    fn build_path(&self) -> Vec<Position> {
        let mut nodes = Vec::new();
        for node in self.previous_nodes.iter() {
            nodes.push(node.position);
        }
        nodes
    }

    pub fn a_star_search(&mut self, map : Map, end_position: Position) -> Vec<Position> {
        let score_estimate = self.manhattan_path_cost(self.start_position, end_position);
        self.f_scores.insert(self.start_position, score_estimate);

        while !self.unvisited.is_empty() {
            let lowest_score = self.unvisited.pop();
            if lowest_score.unwrap().0.position == end_position {
                return self.build_path();
            }

        }

        return Vec::new();
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Reverse;

    use crate::tile::{Tile};
    use crate::room::Room;
    use crate::map;
    use crate::pathfinding;
    use crate::position::{Position, build_square_area};
    use crate::pathfinding::{ManhattanPathCosting, Pathfinding, Node};

    fn build_test_map() -> map::Map {
        let tile_library = crate::tile::build_library();
        assert_eq!(9, tile_library.len());

        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();

        let room_pos = Position { x: 0, y: 0 };
        let room_area = build_square_area(room_pos, 3);
        let doors = Vec::new();
        let room = Room { area: room_area, doors };

        let mut rooms = Vec::new();
        rooms.push(room);

        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);
        let map = crate::map::Map {
            area: map_area,
            tiles : vec![
                vec![ wall.clone(), wall.clone(), wall.clone() ],
                vec![ wall.clone(), rom.clone(), wall.clone() ],
                vec![ wall.clone(), wall.clone(), wall.clone() ],
            ], rooms
        };
        map
    }

    #[test]
    fn test_manhattan_path_cost_zero() {
        // GIVEN 2 positions that are equivalent
        let start_pos = Position { x: 1, y: 3 };
        let end_pos = start_pos.clone();

        // WHEN we call to get the manhattan cost for the distance between these 2 points
        let pathfinding = Pathfinding::build( start_pos);

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
        let pathfinding = Pathfinding::build( start_pos);
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
        let pathfinding = Pathfinding::build( start_pos);
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
        let pathfinding = Pathfinding::build( start_pos);
        let result = pathfinding.manhattan_path_cost(start_pos, end_pos);

        // THEN we expect 4 to be returned
        assert_eq!(4, result);
    }

    #[test]
    fn test_build_unvisited() {
        // GIVEN a starting position
        let start_pos = Position { x: 1, y: 3 };

        // WHEN we build a pathfinding instance
        let pathfinding = Pathfinding::build( start_pos);

        // THEN we expect the unvisited set to contain the start node with a score of 0
        let mut unvisited = pathfinding.unvisited;
        assert_eq!(1, unvisited.len());
        assert_eq!(Node { position: start_pos, score: 0}, unvisited.pop().unwrap().0);
    }

    #[test]
    fn test_a_star_search() {
        // GIVEN a starting position
        let start_pos = Position { x: 0, y: 0 };
        let end_pos = Position { x: 3, y: 0 };

        // AND a valid pathfinding instance
        let mut pathfinding = Pathfinding::build( start_pos);

        // WHEN we call to a* search
        let map = build_test_map();
        let path = pathfinding.a_star_search(map, end_pos);

        // THEN we expect the initial gscore to be 0
        // AND the initial fscore to be the distance between the 2 Positions
        let g_score = pathfinding.g_scores.get(&start_pos);
        assert_eq!(0, *g_score.unwrap());
        let f_score = pathfinding.f_scores.get(&start_pos);
        assert_eq!(3, *f_score.unwrap());
    }
}