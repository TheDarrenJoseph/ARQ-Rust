use uuid::Uuid;
use crate::character::equipment::EquipmentSlot;
use crate::map::objects::items::{Item, ItemType, MaterialType, Weapon};
use crate::map::tile::{Colour, Symbol};

pub struct WeaponBlueprint {
    weapon: Weapon,
    item_type: ItemType,
    material_type: MaterialType,
    name : String,
    symbol : Symbol, // TODO allow using the colour for 'special' weapons?
    weight : i32,
    value : i32,
    equipment_slot: Option<EquipmentSlot>
}

impl WeaponBlueprint {
    
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
        Item::weapon(Uuid::new_v4(), blueprint.name.clone(), blueprint.symbol.character.clone(), blueprint.weight.clone(), blueprint.value.clone(), blueprint.weapon.clone())
    }
}