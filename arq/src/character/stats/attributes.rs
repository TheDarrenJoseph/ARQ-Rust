use std::fmt::{Debug, Display, Formatter, Result};

#[derive(PartialEq, Clone, Debug)]
pub enum Attribute {Strength, Health, Agility, Intelligence, Stealth}

impl Display for Attribute {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, Debug)]
pub struct AttributeScore {
    pub attribute: Attribute,
    pub score: i8
}

pub fn build_default_attributes() -> Vec<AttributeScore> {
    let mut scores = Vec::new();
    let attributes = get_all_attributes();
    for attr in attributes {
        scores.push(AttributeScore { attribute: attr, score: 0 });
    }
    return scores;
}

pub fn get_all_attributes() -> Vec<Attribute> {
    vec![Attribute::Strength, Attribute::Health, Attribute::Agility, Attribute::Intelligence, Attribute::Stealth]
}