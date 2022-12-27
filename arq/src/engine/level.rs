use std::{fmt, io};
use std::io::{Error, ErrorKind};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use termion::event::Key;

use crate::character::Character;
use crate::character::characters::{build_characters, Characters};
use crate::engine::command::input_mapping;
use crate::map::Map;
use crate::map::map_generator::build_generator;
use crate::map::position::{build_rectangular_area, build_square_area, Position, Side};

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

pub fn init_level_manager(rng : Pcg64) -> Levels {
    Levels { rng, levels: vec![], _current_level: 0}
}

pub enum LevelChangeResult {
    LevelChanged,
    OutOfDungeon
}


impl Levels {
    pub fn add_level(&mut self, level: Level) {
        self.levels.push(level);
    }

    pub(crate) fn get_level_mut(&mut self) -> &mut Level {
        return self.levels.get_mut(self._current_level).unwrap();
    }

    pub fn get_current_level(&self) -> usize {
        self._current_level.clone()
    }

    fn build_test_map(&mut self) -> Map {
        let map_size = 12;
        let map_area = build_square_area(Position {x: 0, y: 0}, map_size);
        self.rng = Seeder::from("test".to_string()).make_rng();
        let mut generator = build_generator(&mut self.rng, map_area);
        generator.generate()
    }

    fn build_map(&mut self) -> Map {
        let map_area = build_rectangular_area(Position { x: 0, y: 0 }, 30, 20);
        let rng = &mut self.rng;
        let mut map_generator = build_generator(rng, map_area);
        map_generator.generate()
    }

    pub(crate) fn generate_level(&mut self) {
        let new_level;
        let map = Some(self.build_map());

        let mut player = None;
        if !self.levels.is_empty() {
            // Move the player to the next level
            player = Some(self.get_level_mut().characters.remove_player());
        }
        new_level = Level {
            map,
            characters: build_characters(player, Vec::new() )
        };
        self.levels.push(new_level);
    }

    pub(crate) fn change_level(&mut self, level_change: LevelChange) -> Result<LevelChangeResult, io::Error>  {
        match level_change {
            LevelChange::UP => {
                if self._current_level > 0 {
                    let player = self.get_level_mut().characters.remove_player();
                    self._current_level -= 1;
                    self.get_level_mut().characters.set_player(player);
                    return Ok(LevelChangeResult::LevelChanged);
                } else {
                    return Ok(LevelChangeResult::OutOfDungeon);
                }
            },
            LevelChange::DOWN => {
                if self._current_level < self.levels.len() - 1 {
                    let player = self.get_level_mut().characters.remove_player();
                    self._current_level += 1;
                    self.get_level_mut().characters.set_player(player);
                } else {
                    self.generate_level();
                    self._current_level += 1;
                }
                return Ok(LevelChangeResult::LevelChanged);
            },
            _ => {
            }
        }
        return Ok(LevelChangeResult::LevelChanged);
    }
}

impl Level {
    pub(crate) fn find_adjacent_player_position(&mut self, key: Key) -> Option<Position> {
        return match key {
            Key::Down | Key::Up | Key::Left | Key::Right | Key::Char('w') | Key::Char('a') | Key::Char('s') | Key::Char('d') => {
                if let Some(side) = input_mapping::key_to_side(key) {
                    self.find_player_side_position(side)
                } else {
                    None
                }
            },
            Key::Char(_) => {
                Some(self.characters.get_player_mut().unwrap().get_position().clone())
            }
            _ => {
                None
            }
        };
    }

    pub fn find_player_side_position(&mut self, side: Side) -> Option<Position> {
        let position = self.characters.get_player_mut().unwrap().get_position().clone();
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