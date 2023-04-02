use crate::character::builder::character_builder::{CharacterBuilder, CharacterPattern};
use crate::character::Character;
use crate::map::position::Position;

#[derive(Default, Clone)]
pub struct Characters {
    player: Option<Character>,
    npcs : Vec<Character>
}

pub fn build_empty_characters() -> Characters {
    return Characters { player: None, npcs: Vec::new() };
}

pub fn build_default_characters() -> Characters {
    let position = Position { x: 1, y: 1};
    let player =  CharacterBuilder::new(CharacterPattern::new_player())
        .position(position)
        .build(String::from("Player"));
    return Characters { player: Some(player), npcs: Vec::new() };
}

pub fn build_characters(player: Option<Character>, npcs: Vec<Character>) -> Characters {
    return Characters { player, npcs };
}

impl Characters {
    pub fn get_player(&self) -> Option<&Character> { self.player.as_ref() }
    pub fn get_player_mut(&mut self) -> Option<&mut Character> { self.player.as_mut() }
    pub fn get_npcs(&self) -> &Vec<Character> { &self.npcs }
    pub fn get_npcs_mut(&mut self) -> &mut Vec<Character> { &mut self.npcs }

    pub fn set_player(&mut self, player: Character) {
        self.player = Some(player);
    }

    pub fn set_npcs(&mut self, npcs: Vec<Character>) {
        self.npcs = npcs;
    }

    pub fn remove_player(&mut self) -> Character {
        let player = self.player.clone();
        self.player = None;
        return player.unwrap();
    }
}