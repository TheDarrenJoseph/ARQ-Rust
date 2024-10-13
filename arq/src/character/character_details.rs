use crate::character::stats::attributes::{build_default_attributes, AttributeScore};
use crate::character::{Class, Race};

#[derive(Clone, Debug)]
pub struct CharacterDetails {
    race: Race,
    class: Class,
    level: i32,
    max_free_attribute_points: i8,
    free_attribute_points: i8,
    attributes: Vec<AttributeScore>
}

impl CharacterDetails {
    pub fn get_race(&self) -> &Race {
        &self.race
    }

    pub fn set_race(&mut self, race: Race) {
        self.race = race
    }

    pub fn get_class(&self) -> Class {
        self.class.clone()
    }

    pub fn set_class(&mut self, class: Class) {
        self.class = class;
    }

    pub fn get_level(&self) -> i32 {
        self.level
    }

    pub fn set_level(&mut self, level: i32) {
        self.level = level;
    }

    pub fn get_max_free_attribute_points(&self) -> i8 {
        self.max_free_attribute_points
    }

    pub fn set_max_free_attribute_points(&mut self, max_free_attribute_points: i8) {
        self.max_free_attribute_points = max_free_attribute_points;
    }

    pub fn get_free_attribute_points(&self) -> i8 {
        self.free_attribute_points
    }

    pub fn set_free_attribute_points(&mut self, free_attribute_points: i8) {
        self.free_attribute_points = free_attribute_points;
    }

    pub fn get_attributes(&self) -> Vec<AttributeScore> {
        self.attributes.clone()
    }

    pub fn set_attributes(&mut self, attributes: Vec<AttributeScore>) {
        self.attributes = attributes;
    }
    pub fn new(race: Race, class: Class, level: i32, max_free_attribute_points: i8, free_attribute_points: i8, attributes: Vec<AttributeScore>) -> Self {
        Self { race, class, level, max_free_attribute_points, free_attribute_points, attributes }
    }
}

pub fn build_default_character_details() -> CharacterDetails {
    let attributes = build_default_attributes();
    return CharacterDetails { race: Race::Human, class: Class::None, level:0, max_free_attribute_points: 6, free_attribute_points: 6, attributes};
}