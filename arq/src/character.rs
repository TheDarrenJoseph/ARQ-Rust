use std::fmt::{Debug, Display, Formatter, Result};

use uuid::Uuid;

use crate::map::objects::container::{build, Container, ContainerType};
use crate::map::position::Position;
use crate::map::tile::Colour;

#[derive(Clone, Debug)]
pub struct Character {
    name : String,
    character_details: CharacterDetails,
    health: i8,
    colour: Colour,
    position: Position,
    inventory: Container
}

#[derive(Clone, Debug)]
pub enum Race {Human,Goblin}
#[derive(Clone, Debug)]
pub enum Class {None,Warrior}
#[derive(PartialEq, Clone, Debug)]
pub enum Attribute {Strength, Health, Agility, Intelligence, Stealth}


pub fn determine_class(name: String) -> Option<Class> {
    match name.as_str() {
        "None" => {
            Some(Class::None)
        }
        "Warrior" => {
            Some(Class::Warrior)
        }
        _ => {
            None
        }
    }
}


impl Display for Attribute {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(self, f)
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, Debug)]
pub struct AttributeScore {
    pub attribute: Attribute,
    pub score: i8
}

#[derive(Clone, Debug)]
pub struct CharacterDetails {
    race: Race,
    class: Class,
    level: i32,
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
    return CharacterDetails { race: Race::Human, class: Class::None, level:0, max_free_attribute_points: 6, free_attribute_points: 6, attributes};
}

pub fn build_player(name : String, position: Position) -> Character {
    let health = 100;
    let colour = Colour::Green;
    let inventory = build(Uuid::new_v4(), name.clone() + &"'s Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    let character_details = build_default_character_details();
    let player = Character { name, character_details, health, colour, position, inventory };
    return player;
}

pub fn build_character(name : String, position: Position, inventory: Container) -> Character {
    let health = 100;
    let colour = Colour::Red;
    let character_details = build_default_character_details();
    let player = Character { name, character_details, health, colour, position, inventory };
    return player;
}

impl Character {
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn set_name(&mut self, name : String) {
        self.name = name;
    }

    pub fn get_details(&self) -> CharacterDetails {
        self.character_details.clone()
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

    pub fn set_inventory(&mut self, container: Container) {
       self.inventory = container;
    }

    pub fn get_race(&mut self) -> Race {
        self.character_details.race.clone()
    }

    pub fn set_race(&mut self, race: Race) {
        self.character_details.race = race;
    }

    pub fn get_class(&mut self) -> Class {
        self.character_details.class.clone()
    }

    pub fn set_class(&mut self, class: Class) {
        self.character_details.class = class
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

    pub fn get_attribute_scores(&mut self) -> Vec<AttributeScore> {
        self.character_details.attributes.clone()
    }

    pub fn set_attribute_scores(&mut self, scores : Vec<AttributeScore> ) {
        self.character_details.attributes = scores;
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::character::{build_default_character_details, build_player, Character};
    use crate::map::objects::container::ContainerType;
    use crate::map::position::Position;
    use crate::map::tile::Colour;

    #[test]
    fn test_character_build() {
        let name = String::from("Test Person");
        let character_details = build_default_character_details();
        let health = 100;
        let colour = Colour::Green;
        let position = Position { x: 1, y: 1};
        let inventory = crate::map::objects::container::build(Uuid::new_v4(), "Test Person's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
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
        let inventory = crate::map::objects::container::build(Uuid::new_v4(), "Test Person's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let mut character = build_player(name, position);

        assert_eq!("Test Person", character.get_name());
        assert_eq!(100, character.get_health());
        assert_eq!(Colour::Green, character.get_colour());
        assert_eq!(position, character.get_position());
        assert_eq!(0, character.get_inventory().get_contents().len());
    }
}