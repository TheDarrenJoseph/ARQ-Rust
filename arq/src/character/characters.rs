use crate::character::Character;

#[derive(Default, Debug, Clone)]
pub struct Characters {
    player: Option<Character>,
    npcs : Vec<Character>
}

impl Characters {
    pub fn new(player: Option<Character>, npcs: Vec<Character>) -> Characters {
        Characters { player, npcs }
    }

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