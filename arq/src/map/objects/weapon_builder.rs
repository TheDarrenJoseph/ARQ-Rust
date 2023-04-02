use uuid::Uuid;
use crate::character::equipment::EquipmentSlot;
use crate::map::objects::items::{Item, ItemType, Weapon};
use crate::map::tile::{Colour, Symbol};

pub struct WeaponBlueprint {

}

pub struct WeaponBuilder {
    weapon: Weapon,
    pub item_type: ItemType,
    name : String,
    pub symbol : char,
    pub colour : Colour,
    pub weight : i32,
    pub value : i32,
    equipment_slot: Option<EquipmentSlot>
}
