use std::io;
use rand_pcg::Pcg64;

use termion::event::Key;

use crate::character::Character;
use crate::engine::command::input_mapping;
use crate::map::Map;
use crate::map::map_generator::build_generator;
use crate::map::position::{build_rectangular_area, Position, Side};

#[derive(Default, Clone)]
pub struct Level {
    pub map : Option<Map>,
    pub characters : Characters
}

pub struct Levels {
    pub rng : Pcg64,
    // Implied to always reflect updates to levels
    _current_level: usize,
    levels : Vec<Level>
}

#[derive(Clone)]
pub(crate) enum LevelChange {
    UP,
    DOWN,
    NONE
}

#[derive(Default, Clone)]
pub struct Characters {
    pub characters : Vec<Character>
}

pub fn init_level_manager(rng : Pcg64) -> Levels {
    Levels { rng, levels: vec![], _current_level: 0}
}


impl Levels {
    fn add_level(&mut self, level: Level) {
        self.levels.push(level);
    }

    pub(crate) fn get_level_mut(&mut self) -> &mut Level {
        return self.levels.get_mut(self._current_level).unwrap();
    }

    pub(crate) fn generate_level(&mut self) {
        let map_area = build_rectangular_area(Position { x: 0, y: 0 }, 20, 20);

        let rng = &mut self.rng;
        let mut map_generator = build_generator(rng, map_area);

        let new_level;
        let map = Some(map_generator.generate());
        if !self.levels.is_empty() {
            let player = self.get_level_mut().characters.remove_player();
            new_level = Level {
                map,
                characters: Characters { characters: vec![player] }
            };
        } else {
            new_level = Level {
                map,
                characters: Characters { characters: Vec::new() }
            };
        }

        self.levels.push(new_level);
    }

    pub(crate) fn change_level(&mut self, level_change: LevelChange) -> Result<bool, io::Error>  {
        match level_change {
            LevelChange::UP => {
                if self._current_level > 0 {
                    let player = self.get_level_mut().characters.remove_player();
                    self._current_level -= 1;
                    self.get_level_mut().characters.set_characters(vec![player]);
                    return Ok(true);
                }
            },
            LevelChange::DOWN => {
                if self._current_level < self.levels.len() - 1 {
                    let player = self.get_level_mut().characters.remove_player();
                    self._current_level += 1;
                    self.get_level_mut().characters.set_characters(vec![player]);
                } else {
                    self.generate_level();
                    self._current_level += 1;
                }
                return Ok(true);
            },
            _ => {
            }
        }
        return Ok(false);
    }
}


impl  Characters {
    pub fn get_player(&self) -> &Character {
        &self.characters[0]
    }

    pub fn remove_player(&mut self) -> Character {
        self.characters.remove(0)
    }

    pub(crate) fn get_player_mut(&mut self) -> &mut Character {
        &mut self.characters[0]
    }

    pub fn set_characters(&mut self, characters: Vec<Character>) {
        self.characters = characters;
    }
}

impl Level {
    pub(crate) fn find_adjacent_player_position(&mut self, key: Key, _command_char: Key) -> Option<Position> {
        return match key {
            Key::Down | Key::Up | Key::Left | Key::Right => {
                if let Some(side) = input_mapping::key_to_side(key) {
                    self.find_player_side_position(side)
                } else {
                    None
                }
            },
            Key::Char(_) => {
                Some(self.characters.get_player_mut().get_position().clone())
            }
            _ => {
                None
            }
        };
    }

    pub fn find_player_side_position(&mut self, side: Side) -> Option<Position> {
        let position = self.characters.get_player_mut().get_position().clone();
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

    pub fn get_map(&self) -> &Option<Map> {
        &self.map
    }

    pub fn set_map(&mut self, map : Option<Map>) {
        self.map = map
    }

    pub fn get_map_mut(&mut self) -> Option<&mut Map> {
        self.map.as_mut()
    }
}