pub mod characters;
pub mod character_details;
pub mod stats;
pub mod equipment;

use std::fmt::{Debug, Display, Formatter, Result};

use uuid::Uuid;
use crate::character::character_details::{build_default_character_details, CharacterDetails};
use crate::character::equipment::Equipment;
use crate::character::stats::attributes::AttributeScore;

use crate::map::objects::container::{build, Container, ContainerType};
use crate::map::position::Position;
use crate::map::tile::Colour;

#[derive(Copy, Clone, Debug)]
pub enum Race {Human,Goblin}

#[derive(Copy, Clone, Debug)]
pub enum Class {None,Warrior}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, Debug)]
pub struct Character {
    name : String,
    character_details: CharacterDetails,
    health: i8,
    colour: Colour,
    position: Position,
    inventory: Container,
    equipment: Equipment
}

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

pub fn build_player(name : String, position: Position) -> Character {
    let inventory = build(Uuid::new_v4(), name.clone() + &"'s Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    return build_character(name, position, Colour::Green, inventory)
}

pub fn build_character(name : String, position: Position, colour: Colour, inventory: Container) -> Character {
    let health = 100;
    let character_details = build_default_character_details();
    let equipment = Equipment::new();
    let player = Character { name, character_details, health, colour, position, inventory, equipment };
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

    pub fn get_inventory_mut(&mut self) -> &mut Container {
        return &mut self.inventory;
    }

    pub fn set_inventory(&mut self, container: Container) {
       self.inventory = container;
    }

    pub fn set_equipment(&mut self, equipment: Equipment) {
        self.equipment = equipment
    }

    pub fn get_equipment_mut(&mut self) -> &mut Equipment {
        return &mut self.equipment;
    }

    pub fn get_race(&mut self) -> Race {
        self.character_details.get_race().clone()
    }

    pub fn set_race(&mut self, race: Race) {
        self.character_details.set_race(race);
    }

    pub fn get_class(&mut self) -> Class {
        self.character_details.get_class()
    }

    pub fn set_class(&mut self, class: Class) {
        self.character_details.set_class(class)
    }

    pub fn get_max_free_attribute_points(&mut self) -> i8 {
        self.character_details.get_max_free_attribute_points()
    }

    pub fn get_free_attribute_points(&mut self) -> i8 {
        self.character_details.get_free_attribute_points()
    }

    pub fn set_free_attribute_points(&mut self, points: i8) {
        self.character_details.set_free_attribute_points(points);
    }

    pub fn get_attribute_scores(&mut self) -> Vec<AttributeScore> {
        self.character_details.get_attributes()
    }

    pub fn set_attribute_scores(&mut self, scores : Vec<AttributeScore> ) {
        self.character_details.set_attributes(scores);
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::character::{build_player, Character};
    use crate::character::character_details::build_default_character_details;
    use crate::character::equipment::Equipment;
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
        let equipment = Equipment::new();
        let mut character = Character { name, character_details, health, colour, position, inventory, equipment };

        assert_eq!("Test Person", character.get_name());
        assert_eq!(100, character.get_health());
        assert_eq!(Colour::Green, character.get_colour());
        assert_eq!(position, character.get_position());
        assert_eq!(0, character.get_inventory_mut().get_contents().len());
    }


    #[test]
    fn test_build_player() {
        let name = String::from("Test Person");
        let _health = 100;
        let _colour = Colour::Green;
        let position = Position { x: 1, y: 1};
        let _inventory = crate::map::objects::container::build(Uuid::new_v4(), "Test Person's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let mut character = build_player(name, position);

        assert_eq!("Test Person", character.get_name());
        assert_eq!(100, character.get_health());
        assert_eq!(Colour::Green, character.get_colour());
        assert_eq!(position, character.get_position());
        assert_eq!(0, character.get_inventory_mut().get_contents().len());
    }
}
