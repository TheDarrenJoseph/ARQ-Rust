use uuid::Uuid;
use crate::character::{Character, Class, Race};
use crate::character::builder::character_builder::CharacterType::{GoblinWarrior, NewPlayer};
use crate::character::character_details::{build_default_character_details, CharacterDetails};
use crate::character::equipment::Equipment;
use crate::character::stats::attributes::{AttributeScore, AttributeScores};
use crate::map::map_generator::build_dev_inventory;
use crate::map::objects::container;
use crate::map::objects::container::{build, Container, ContainerType};
use crate::map::position::Position;
use crate::map::tile::{Colour, Symbol};

const DEFAULT_POSITION: Position = Position { x: 0, y: 0 };

#[derive(Clone, Debug)]
pub enum CharacterType {
    NewPlayer, // Default human player character, before character building / specialisation
    GoblinWarrior
}

#[derive(Clone, Debug)]
pub struct CharacterPattern {
    character_type: CharacterType,
    blueprint: CharacterBlueprint
}

/*
    Future TODO - Consider storing patterns in a proper data store so we can look them up by CharacterType alone / keep the code lightweight
 */
impl CharacterPattern {
    pub fn player() -> CharacterPattern {
        let attributes: Vec<AttributeScore> = AttributeScores::default().scores;
        let blueprint = CharacterBlueprint {
            details : CharacterDetails::new(Race::Human, Class::None, 0, 6, 6, attributes),
            position: None,
            symbol: Symbol::new('@', Colour::Green),
            health: 100,
            inventory: build_dev_inventory(),
            equipment: Equipment::new()
        };

        CharacterPattern { character_type: NewPlayer, blueprint }
    }

    pub fn goblin() -> CharacterPattern {
        let inventory = build(Uuid::new_v4(), "A Goblin's dead body".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let attributes: Vec<AttributeScore> = AttributeScores::all_at_value(2).scores;
        let blueprint = CharacterBlueprint {
            details : CharacterDetails::new(Race::Goblin, Class::None, 1, 3, 0, attributes),
            position: None,
            symbol: Symbol::new('g', Colour::Green),
            health: 80,
            inventory,
            equipment: Equipment::new()
        };

        CharacterPattern { character_type: GoblinWarrior, blueprint }
    }
}

/*
    These are the more generally re-usable elements of character patterns that can be used to generate individuals
 */
#[derive(Clone, Debug)]
pub struct CharacterBlueprint {
    pub details: CharacterDetails,
    pub position: Option<Position>,
    pub symbol: Symbol,
    pub health: i8,
    pub inventory: Container,
    pub equipment: Equipment
}

pub struct CharacterBuilder {
    pattern: CharacterPattern
}

impl CharacterBuilder {
    pub fn new(character_pattern: CharacterPattern) -> CharacterBuilder {
        CharacterBuilder { pattern: character_pattern }
    }

    pub fn position(&mut self, position: Position) -> &mut CharacterBuilder {
        self.pattern.blueprint.position = Some(position);
        self
    }

    pub fn build(&self, character_name: String) -> Character {
        let character_type = &self.pattern.character_type;
        let blueprint = &self.pattern.blueprint;
        let details = &blueprint.details;
        let symbol = &blueprint.symbol;
        let health = blueprint.health;
        let mut inventory = blueprint.inventory.clone();

        match character_type {
            NewPlayer => {
                let container_name = format!("{}'s Inventory", character_name);
                inventory.get_self_item_mut().set_name(container_name);
            },
            _ => {}
        }

        let position = if let Some(pos) = blueprint.position {
            pos
        } else {
            DEFAULT_POSITION
        };

        let equipment = &blueprint.equipment;
        return Character::new_detailed(character_name, position, details.clone(), symbol.clone(), health, inventory.clone(), equipment.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::character::builder::character_builder::{CharacterBuilder, CharacterPattern, DEFAULT_POSITION};
    use crate::map::tile::Colour;

    #[test]
    pub fn test_build_player() {
        // GIVEN a CharacterBuilder with the player pattern
        let builder = CharacterBuilder::new(CharacterPattern::player());
        // WHEN we calll to build a character
        let mut character = builder.build(String::from("Player"));

        assert_eq!("Player", character.name);
        assert_eq!('@', character.symbol.symbol);
        assert_eq!(Colour::Green, character.symbol.colour);

        // TODO check details
        let details = character.character_details.clone();

        assert_eq!(100, character.health);
        assert_eq!(DEFAULT_POSITION, character.position);

        let inventory = character.get_inventory_mut();
        assert_eq!(62, inventory.get_contents().len());

        let equipment = character.get_equipment_mut();
        assert_eq!(0, equipment.get_slots().len())
    }
}
