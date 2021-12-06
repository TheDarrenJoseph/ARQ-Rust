use std::collections::{BinaryHeap,HashMap};
use std::cmp::{Reverse,Ordering};

use crate::map::Map;
use crate::map::position::Position;

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

pub struct Pathfinding {
    start_position: Position,
    unvisited : BinaryHeap<Reverse<Node>>,
    came_from: HashMap<Position, Position>,
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
        let mut pathfinding = Pathfinding { start_position, unvisited: BinaryHeap::new(), came_from: HashMap::new(), g_scores: HashMap::new(), f_scores: HashMap::new() };
        let node = Node { position: start_position, score: 0 };
        let reversed = Reverse(node);
        pathfinding.unvisited.push(reversed);
        pathfinding.g_scores.insert(start_position, 0);
        pathfinding
    }

    fn build_path(&self, position: Position) -> Vec<Position> {
        let mut nodes = Vec::new();
        nodes.push(position);

        let mut next_node = position;
        let mut has_next_node = true;
        while has_next_node {
            let previous_node = self.came_from.get(&next_node);

            match previous_node {
                Some(n) => {
                    next_node = *n;
                    nodes.push(*n);
                },
                None => {
                    has_next_node = false;
                }
            }
        }
        nodes.reverse();
        nodes
    }

    fn get_g_score(&self, position: Position) -> i32 {
        *self.g_scores.get(&position).unwrap_or(&(i16::MAX as i32))
    }

    fn position_unvisited(&self, position: Position) -> bool {
        match self.unvisited.iter().find(|u| u.0.position == position) {
            Some(_) => {
                return true;
            },
            _ => {
                return false;
            }
        }
    }

    pub fn a_star_search(&mut self, map : &Map, end_position: Position) -> Vec<Position> {
        let score_estimate = self.manhattan_path_cost(self.start_position, end_position);
        self.f_scores.insert(self.start_position, score_estimate);

        while !self.unvisited.is_empty() {
            let current_lowest_score_node = self.unvisited.pop().unwrap().0;
            if current_lowest_score_node.position == end_position {
                log::info!("Found end position {:?}", end_position);
                return self.build_path(end_position);
            }

            if map.is_paveable(current_lowest_score_node.position) {
                let neighbors = current_lowest_score_node.position.get_neighbors();
                log::info!("Found {} neighbors for current_lowest_score_node: {:?}", neighbors.len(), current_lowest_score_node.position);

                let current_position = current_lowest_score_node.position.clone();
                let current_g_score = self.get_g_score(current_position);
                log::info!("Current current_lowest_score_node gScore {}", current_g_score);
                for n in neighbors {
                    log::info!("Evaluating neighbor {:?}", n);
                    let neighbor_g_score = self.g_scores.get(&n).unwrap_or(&(i16::MAX as i32));
                    log::info!("Current neighbor gScore {}", current_g_score);
                    let distance_through = self.manhattan_path_cost(current_position, n);
                    log::info!("Distance through neighbor {}", distance_through);
                    let potential_g_score = current_g_score + distance_through;
                    if potential_g_score < *neighbor_g_score {
                        self.came_from.insert(n, current_position);
                        self.g_scores.insert(n, potential_g_score);
                        let through_neighbor_score = self.manhattan_path_cost(n, end_position);

                        let neighbor_fscore = potential_g_score + through_neighbor_score;
                        self.f_scores.insert(n, neighbor_fscore);

                        if !self.position_unvisited(n) {
                            let node = Node { position: n, score: neighbor_fscore };
                            self.unvisited.push(Reverse(node));
                        }
                    }
                }
            }
        }

        return Vec::new();
    }
}

#[cfg(test)]
mod tests {
    use crate::map::tile::{Tile};
    use crate::map::room::Room;
    use crate::map;
    use crate::map::position::{Position, build_square_area};
    use crate::pathfinding::{ManhattanPathCosting, Pathfinding, Node};

    fn build_test_map() -> map::Map {
        let tile_library = crate::map::tile::build_library();
        assert_eq!(9, tile_library.len());

        let non = tile_library[&Tile::NoTile].clone();
        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();
        let door = tile_library[&Tile::Door].clone();

        let room_pos = Position { x: 0, y: 0 };
        let room_area = build_square_area(room_pos, 3);
        let doors = Vec::new();
        let room = Room { area: room_area, doors };

        let mut rooms = Vec::new();
        rooms.push(room);

        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 4);
        let map = crate::map::Map {
            area: map_area,
            tiles : vec![
                vec![ non.clone(), non.clone(), non.clone(), non.clone(), ],
                vec![ non.clone(), wall.clone(), wall.clone(), wall.clone(), ],
                vec![ non.clone(), door.clone(), rom.clone(), wall.clone(), ],
                vec![ non.clone(), wall.clone(), wall.clone(), wall.clone(), ],
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
    fn test_a_star_search_straight_line_no_obstacles() {
        // GIVEN a starting position
        let start_pos = Position { x: 0, y: 0 };
        let end_pos = Position { x: 3, y: 0 };

        // AND a valid pathfinding instance
        let mut pathfinding = Pathfinding::build( start_pos);

        // WHEN we call to a* search in a straight line with no obstacles
        let map = build_test_map();
        let path = pathfinding.a_star_search(&map, end_pos);

        // THEN we expect the initial gscore to be 0
        // AND the initial fscore to be the distance between the 2 Positions
        let g_score = pathfinding.g_scores.get(&start_pos);
        assert_eq!(0, *g_score.unwrap());
        let f_score = pathfinding.f_scores.get(&start_pos);
        assert_eq!(3, *f_score.unwrap());

        // AND the path to be 4 nodes long
        assert_eq!(4, path.len());
        // AND look like this
        assert_eq!(Position{x:0, y:0}, *path.get(0).unwrap());
        assert_eq!(Position{x:1, y:0}, *path.get(1).unwrap());
        assert_eq!(Position{x:2, y:0}, *path.get(2).unwrap());
        assert_eq!(Position{x:3, y:0}, *path.get(3).unwrap());
    }

    #[test]
    fn test_a_star_search_obstacles_into_room() {
        // GIVEN a starting position
        let start_pos = Position { x: 0, y: 0 };
        // AND an end position targeting a Room tile inside a room
        let end_pos = Position { x: 2, y: 2 };

        // AND a valid pathfinding instance
        let mut pathfinding = Pathfinding::build( start_pos);

        // WHEN we call to a* search in a straight line with no obstacles
        let map = build_test_map();
        let path = pathfinding.a_star_search(&map, end_pos);

        // THEN we expect the initial gscore to be 0
        // AND the initial fscore to be the distance between the 2 Positions
        let g_score = pathfinding.g_scores.get(&start_pos);
        assert_eq!(0, *g_score.unwrap());
        let f_score = pathfinding.f_scores.get(&start_pos);
        assert_eq!(4, *f_score.unwrap());

        // AND the path to be 5 nodes long
        assert_eq!(5, path.len());
        // AND look like this
        assert_eq!(Position{x:0, y:0}, *path.get(0).unwrap());
        assert_eq!(Position{x:0, y:1}, *path.get(1).unwrap());
        assert_eq!(Position{x:0, y:2}, *path.get(2).unwrap());
        assert_eq!(Position{x:1, y:2}, *path.get(3).unwrap());
        assert_eq!(Position{x:2, y:2}, *path.get(4).unwrap());
    }
}