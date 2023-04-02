use uuid::Uuid;
use crate::character::equipment::EquipmentSlot;

use crate::map::tile::{Colour, Symbol};

#[derive(Clone, PartialEq, Debug)]
pub enum ItemType {
    ITEM,
    CONTAINER,
    WEAPON(Weapon),
    HEADGEAR,
    TORSO,
    LEGS
}

#[derive(Clone, PartialEq, Debug)]
pub struct Weapon {
    pub damage : i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    id : Uuid,
    pub item_type: ItemType,
    name : String,
    pub symbol : Symbol,
    pub weight : i32,
    pub value : i32,
    equipment_slot: Option<EquipmentSlot>
}

impl Item {
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_weight(&self) -> i32 {
        self.weight.clone()
    }
    pub fn get_value(&self) -> i32 {
        self.value.clone()
    }
    pub fn is_container(&self) -> bool {
        self.item_type == ItemType::CONTAINER
    }
    pub fn is_equipped(&self) -> bool { self.equipment_slot.is_some() }
    pub fn id_equals(&self, other: &Item) -> bool {
        self.id == other.id
    }

    pub fn set_equipment_slot(&mut self, slot: Option<EquipmentSlot>) {
      self.equipment_slot = slot
    }

    pub fn get_equipment_slot(&self) -> Option<EquipmentSlot> {
        self.equipment_slot.clone()
    }

    pub fn unequip(&mut self) {
        self.equipment_slot = None;
    }
}

pub fn build_item(id: Uuid, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id, item_type: ItemType::ITEM, name, symbol: Symbol::new(symbol, Colour::White), weight, value, equipment_slot: None }
}

pub fn build_weapon(id: Uuid, name: String, symbol: char, weight : i32, value : i32, weapon: Weapon) -> Item {
    Item {id, item_type: ItemType::WEAPON(weapon), name, symbol: Symbol::new(symbol, Colour::White), weight, value, equipment_slot: None }
}

pub fn build_container_item(id: Uuid, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id, item_type: ItemType::CONTAINER, name, symbol: Symbol::new(symbol, Colour::White), weight, value, equipment_slot: None }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::map::objects::items;
    use crate::map::tile::Colour;

    #[test]
    fn test_build_item() {
        let id = Uuid::new_v4();
        let item = items::build_item(id, "Test Item".to_owned(), 'X', 1, 1);
        assert_eq!(id, item.get_id());
        assert_eq!(items::ItemType::ITEM, item.item_type);
        assert_eq!("Test Item", item.name);
        assert_eq!('X', item.symbol.character);
        assert_eq!(Colour::White, item.symbol.colour);
        assert_eq!(1, item.weight);
        assert_eq!(1, item.value);
    }

    #[test]
    fn test_build_container() {
        let id = Uuid::new_v4();
        let item = items::build_container_item(id, "Test Container".to_owned(), 'X', 1, 1);

        assert_eq!(id, item.get_id());
        assert_eq!(items::ItemType::CONTAINER, item.item_type);
        assert_eq!("Test Container", item.name);
        assert_eq!('X', item.symbol.character);
        assert_eq!(Colour::White, item.symbol.colour);
        assert_eq!(1, item.weight);
        assert_eq!(1, item.value);
    }
}