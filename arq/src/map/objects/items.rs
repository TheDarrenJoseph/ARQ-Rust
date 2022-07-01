use uuid::Uuid;

use crate::map::tile::Colour;

#[derive(Clone)]
#[derive(PartialEq, Debug)]
pub enum ItemType {
    ITEM,
    CONTAINER,
    WEAPON,
    HEADGEAR,
    TORSO,
    LEGS
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    id : Uuid,
    pub item_type: ItemType,
    pub name : String,
    pub symbol : char,
    pub colour : Colour,
    pub weight : i32,
    pub value : i32
}

impl Item {
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
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
}

pub fn build_item(id: Uuid, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id: id, item_type: ItemType::ITEM, name : name, symbol : symbol, colour: Colour::White, weight: weight, value: value}
}

pub fn build_container_item(id: Uuid, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id: id, item_type: ItemType::CONTAINER, name : name, symbol : symbol, colour: Colour::White, weight: weight, value: value}
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
        assert_eq!('X', item.symbol);
        assert_eq!(Colour::White, item.colour);
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
        assert_eq!('X', item.symbol);
        assert_eq!(Colour::White, item.colour);
        assert_eq!(1, item.weight);
        assert_eq!(1, item.value);
    }
}