use std::error::Error;
use std::fmt::format;
use uuid::Uuid;
use crate::character::equipment::EquipmentSlot;
use crate::error::errors::GenericError;
use crate::map::objects::items::{Item, ItemForm, ItemType, MaterialType, Weapon};
use crate::map::tile::{Colour, Symbol};

pub struct WeaponBlueprint {
    weapon: Weapon,
    item_type: ItemType,
    item_form: ItemForm,
    material_type: MaterialType,
    name : String,
    symbol : Symbol, // TODO allow using the colour for 'special' weapons?
    weight : i32,
    value : i32,
    equipment_slot: Option<EquipmentSlot>
}

impl WeaponBlueprint {
    /* Abstract damage amount */
    fn determine_damage(material_type: MaterialType) -> Result<i32, GenericError>  {
        return match material_type {
            MaterialType::IRON => {
                Ok(20)
            },
            MaterialType::STEEL => {
                Ok(30)
            },
            MaterialType::SILVER => {
                Ok(18)
            },
            MaterialType::GOLD => {
                Ok(10)
            },
            // TODO potentially allow UNKNOWN types, but we'd need to provide data for it's properties as a weapon
            _ => {
                Err(GenericError::new(format!("Unsupported material type for a weapon: {:?}", material_type)))
            }
        }
    }

    pub fn new(material_type: MaterialType, item_form: ItemForm) -> Result<WeaponBlueprint, GenericError> {
        let material_strength = Self::determine_damage(material_type.clone())?;
        let density_cm3 = &material_type.density_cm3();
        Ok(WeaponBlueprint {
            weapon: Weapon {
                damage: material_strength
            },
            item_type: ItemType::ITEM,
            item_form,
            material_type,
            name: "".to_string(), // Default to empty name
            symbol: Symbol { character: '|', colour: Colour::White},
            weight: 1,
            value: 1,
            equipment_slot: None
        })
    }
}

pub struct WeaponBuilder {
    blueprint: WeaponBlueprint
}

impl WeaponBuilder {
    pub fn new(blueprint : WeaponBlueprint) -> WeaponBuilder {
        WeaponBuilder { blueprint }
    }

    pub fn build(&self) -> Item {
        let blueprint = &self.blueprint;
        Item::weapon(Uuid::new_v4(), blueprint.name.clone(), ItemForm::OTHER(blueprint.name.clone()), blueprint.material_type.clone(), blueprint.symbol.character.clone(), blueprint.weight.clone(), blueprint.value.clone(), blueprint.weapon.clone())
    }
}