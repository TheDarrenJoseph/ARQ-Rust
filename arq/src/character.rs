use crate::container::{Container,ContainerType};
use crate::position::Position;
use crate::tile::Colour;

#[derive(Clone)]
pub struct Character {
    name : String,
    health: i8,
    colour: Colour,
    position: Position,
    inventory: Container
}

pub fn build_player(name : String, position: Position) -> Character {
    let health = 100;
    let colour = Colour::Green;
    let inventory = crate::container::build(0, name.clone() + &"'s Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
    let player = Character { name, health, colour, position, inventory };
    return player;
}

impl Character {
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn get_health(&self) -> i8 {
        return self.health.clone();
    }

    pub fn set_health(&mut self, health: i8)  {
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

}

#[cfg(test)]
mod tests {
    use crate::character::{Character, build_player};
    use crate::container::{Container, ContainerType};
    use crate::tile::Colour;
    use crate::position::Position;

    #[test]
    fn test_character_build() {
        let name = String::from("Test Person");
        let health = 100;
        let colour = Colour::Green;
        let position = Position { x: 1, y: 1};
        let inventory = crate::container::build(0, "Test Person's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let mut character = Character { name, health, colour, position, inventory };

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