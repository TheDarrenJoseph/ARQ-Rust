use std::fmt::{Formatter, Display, Result, Debug};

use crate::container::{Container,ContainerType};
use crate::position::Position;
use crate::tile::Colour;

#[derive(Clone)]
pub struct Character {
    name : String,
    character_details: CharacterDetails,
    health: i8,
    colour: Colour,
    position: Position,
    inventory: Container
}

#[derive(Clone)]
pub enum Race {Human,Goblin}
#[derive(Clone)]
pub enum Class {None,Warrior}
#[derive(Clone, Debug)]
pub enum Attribute {Strength, Health, Agility, Intelligence, Stealth}

impl Display for Attribute {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone)]
pub struct AttributeScore {
    attribute: Attribute,
    score: i8
}

#[derive(Clone)]
pub struct CharacterDetails {
    race: Race,
    class: Class,
    max_free_attribute_points: i8,
    free_attribute_points: i8,
    attributes: Vec<AttributeScore>
}
pub fn get_all_attributes() -> Vec<Attribute> {
    vec![Attribute::Strength, Attribute::Health, Attribute::Agility, Attribute::Intelligence, Attribute::Stealth]
}

pub fn build_default_attributes() -> Vec<AttributeScore> {
    let mut scores = Vec::new();
    let attributes = get_all_attributes();
    for attr in attributes {
        scores.push(AttributeScore { attribute: attr, score: 0 });
    }
    return scores;
}

pub fn build_default_character_details() -> CharacterDetails{
    let attributes = build_default_attributes();
    return CharacterDetails { race: Race::Human, class: Class::None, max_free_attribute_points: 6, free_attribute_points: 6, attributes};
}

pub fn build_player(name : String, position: Position) -> Character {
    let health = 100;
    let colour = Colour::Green;
    let inventory = crate::container::build(0, name.clone() + &"'s Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    let character_details = build_default_character_details();
    let player = Character { name, character_details, health, colour, position, inventory };
    return player;
}

impl Character {
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn get_health(&self) -> i8 {
        return self.health.clone();
    }

    pub fn set_health(&mut self, health: i8) {
        self.health = health;
    }

    pub fn get_colour(&self) -> Colour {
        return self.colour.clone();
    }

    pub fn get_position(&self) -> Position {
        return self.position.clone();
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn get_inventory(&mut self) -> &mut Container {
        return &mut self.inventory;
    }

    pub fn get_max_free_attribute_points(&mut self) -> i8 {
        self.character_details.max_free_attribute_points
    }

    pub fn get_free_attribute_points(&mut self) -> i8 {
        self.character_details.free_attribute_points
    }

    pub fn set_free_attribute_points(&mut self, points: i8) {
        self.character_details.free_attribute_points = points;
    }
}

#[cfg(test)]
mod tests {
    use crate::character::{Character, build_player, build_default_character_details};
    use crate::container::{ContainerType};
    use crate::tile::Colour;
    use crate::position::Position;

    #[test]
    fn test_character_build() {
        let name = String::from("Test Person");
        let character_details = build_default_character_details();
        let health = 100;
        let colour = Colour::Green;
        let position = Position { x: 1, y: 1};
        let inventory = crate::container::build(0, "Test Person's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let mut character = Character { name, character_details, health, colour, position, inventory };

        assert_eq!("Test Person", character.get_name());
        assert_eq!(100, character.get_health());
        assert_eq!(Colour::Green, character.get_colour());
        assert_eq!(position, character.get_position());
        assert_eq!(0, character.get_inventory().get_contents().len());
    }


    #[test]
    fn test_build_player() {
        let name = String::from("Test Person");
        let health = 100;
        let colour = Colour::Green;
        let position = Position { x: 1, y: 1};
        let inventory = crate::container::build(0, "Test Person's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let mut character = build_player(name, position);

        assert_eq!("Test Person", character.get_name());
        assert_eq!(100, character.get_health());
        assert_eq!(Colour::Green, character.get_colour());
        assert_eq!(position, character.get_position());
        assert_eq!(0, character.get_inventory().get_contents().len());
    }
}