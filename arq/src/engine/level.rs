use crate::map::Map;
use crate::character::Character;
use crate::map::position::{Side, Position};

pub struct Level {
    pub map : Option<Map>,
    pub characters : Vec<Character>
}

impl Level {
    pub fn find_player_side_position(&mut self, side: Side) -> Option<Position> {
        let position = self.get_player_mut().get_position().clone();
        let mut side_position = None;
        match side {
            Side::TOP => {
                if position.y > 0 {
                    side_position = Some(Position { x: position.x, y: position.y - 1 });
                }
            },
            Side::BOTTOM => {
                side_position = Some(Position { x: position.x, y: position.y + 1 });
            },
            Side::LEFT => {
                if position.x > 0 {
                    side_position = Some(Position { x: position.x - 1, y: position.y });
                }
            },
            Side::RIGHT => {
                side_position = Some(Position { x: position.x + 1, y: position.y });
            }
        }
        side_position
    }

    pub(crate) fn get_map(&self) -> &Option<Map> {
        &self.map
    }

    fn set_map(&mut self, map : Option<Map>) {
        self.map = map
    }

    pub fn get_player(&self) -> &Character {
        &self.characters[0]
    }

    pub(crate) fn get_player_mut(&mut self) -> &mut Character {
        &mut self.characters[0]
    }

    pub fn set_characters(&mut self, characters: Vec<Character>) {
        self.characters = characters;
    }
}