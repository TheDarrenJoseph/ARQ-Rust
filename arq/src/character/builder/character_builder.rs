use uuid::Uuid;
use crate::character::{Character, Class, Race};
use crate::character::builder::character_builder::CharacterType::{GoblinWarrior, NewPlayer};
use crate::character::character_details::{build_default_character_details, CharacterDetails};
use crate::character::equipment::Equipment;
use crate::character::stats::attributes::{AttributeScore, AttributeScores};
use crate::map::map_generator::build_dev_player_inventory;
use crate::map::objects::{container, items};
use crate::map::objects::container::{build, Container, ContainerType};
use crate::map::objects::items::{Item, ItemForm, MaterialType, Weapon};
use crate::map::objects::weapon_builder::SwordType;
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
    pub fn new_player() -> CharacterPattern {
        let attributes: Vec<AttributeScore> = AttributeScores::default().scores;
        let steel_sword = Item::weapon(Uuid::new_v4(), "".to_owned(), ItemForm::SWORD(SwordType::ARMING), MaterialType::STEEL, 'X', 3.0, 50, Weapon { damage: 20 });

        let inventory = build_dev_player_inventory();
        let blueprint = CharacterBlueprint {
            details : CharacterDetails::new(Race::Human, Class::None, 0, 6, 6, attributes),
            position: None,
            symbol: Symbol::new('@', Colour::Green),
            health: 100,
            inventory,
            equipment: Equipment::new()
        };

        CharacterPattern { character_type: NewPlayer, blueprint }
    }

    pub fn goblin() -> CharacterPattern {
        //let steel_short_sword = items::build_weapon(Uuid::new_v4(), "Steel Short Sword".to_owned(), 'X', 3, 50);

        let inventory = build(Uuid::new_v4(), "A Goblin's dead body".to_owned(), 'X', 1.0, 1,  ContainerType::OBJECT, 100);
        let attributes: Vec<AttributeScore> = AttributeScores::all_at_value(2).scores;
        let blueprint = CharacterBlueprint {
            details : CharacterDetails::new(Race::Goblin, Class::Warrior, 1, 3, 0, attributes),
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
    use crate::character::{Class, Race};
    use crate::character::stats::attributes::{AttributeScore, AttributeScores};
    use crate::map::tile::Colour;

    fn assert_attribs(expected: Vec<AttributeScore>, actual: Vec<AttributeScore>) {
        assert_eq!(expected.len(), actual.len());
        for e in expected {
            let found = actual.iter().find(|a| a.attribute == e.attribute);
            assert!(found.is_some());
            assert_eq!(e.score, found.unwrap().score)
        }
    }

    #[test]
    pub fn test_build_new_player() {
        // GIVEN a CharacterBuilder with the player pattern
        let builder = CharacterBuilder::new(CharacterPattern::new_player());
        // WHEN we calll to build a character
        let mut character = builder.build(String::from("Player"));

        assert_eq!("Player", character.name);
        assert_eq!('@', character.symbol.character);
        assert_eq!(Colour::Green, character.symbol.colour);

        // TODO check details
        let details = character.character_details.clone();
        assert_eq!(Race::Human, details.get_race().clone());
        assert_eq!(Class::None, details.get_class().clone());
        let attribs = details.get_attributes();
        let expected_attribs = AttributeScores::all_at_value(0).scores;
        assert_attribs(expected_attribs, attribs);
        assert_eq!(0, details.get_level());
        assert_eq!(6, details.get_free_attribute_points());
        assert_eq!(6, details.get_max_free_attribute_points());

        assert_eq!(100, character.health);
        assert_eq!(DEFAULT_POSITION, character.position);

        let inventory = character.get_inventory_mut();
        assert_eq!(61, inventory.get_contents().len());

        let equipment = character.get_equipment_mut();
        assert_eq!(0, equipment.get_slots().len())
    }

    #[test]
    pub fn test_build_goblin() {
        // GIVEN a CharacterBuilder with the goblin pattern
        let builder = CharacterBuilder::new(CharacterPattern::goblin());
        // WHEN we calll to build a character
        let mut character = builder.build(String::from("Ruggo"));

        assert_eq!("Ruggo", character.name);
        assert_eq!('g', character.symbol.character);
        assert_eq!(Colour::Green, character.symbol.colour);

        // TODO check details
        let details = character.character_details.clone();
        assert_eq!(Race::Goblin, details.get_race().clone());
        assert_eq!(Class::Warrior, details.get_class().clone());
        let attribs = details.get_attributes();
        let expected_attribs = AttributeScores::all_at_value(2).scores;
        assert_attribs(expected_attribs, attribs);
        assert_eq!(1, details.get_level());
        assert_eq!(0, details.get_free_attribute_points());
        assert_eq!(3, details.get_max_free_attribute_points());

        assert_eq!(80, character.health);
        assert_eq!(DEFAULT_POSITION, character.position);

        let inventory = character.get_inventory_mut();
        assert_eq!(0, inventory.get_contents().len());

        let equipment = character.get_equipment_mut();
        assert_eq!(0, equipment.get_slots().len())
    }
}
