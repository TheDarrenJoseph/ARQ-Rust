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

#[derive(Clone, Debug)]
pub struct Item {
    id : u64,
    pub item_type: ItemType,
    pub name : String,
    pub symbol : char,
    pub colour : i32,
    pub weight : i32,
    pub value : i32
}

impl Item {
    pub fn get_id(&self) -> u64 {
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
}

pub fn build_item(id: u64, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id: id, item_type: ItemType::ITEM, name : name, symbol : symbol, colour: 0, weight: weight, value: value}
}

pub fn build_container_item(id: u64, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id: id, item_type: ItemType::CONTAINER, name : name, symbol : symbol, colour: 0, weight: weight, value: value}
}

#[cfg(test)]
mod tests {
    use crate::items::ItemType;

    #[test]
    fn test_build_item() {
        let item = crate::items::build_item(0, "Test Item".to_owned(), 'X', 1, 1);

        assert_eq!(0, item.get_id());
        assert_eq!(ItemType::ITEM, item.item_type);
        assert_eq!("Test Item", item.name);
        assert_eq!('X', item.symbol);
        assert_eq!(0, item.colour);
        assert_eq!(1, item.weight);
        assert_eq!(1, item.value);
    }

    #[test]
    fn test_build_container() {
        let item = crate::items::build_container_item(0, "Test Container".to_owned(), 'X', 1, 1);

        assert_eq!(0, item.get_id());
        assert_eq!(ItemType::CONTAINER, item.item_type);
        assert_eq!("Test Container", item.name);
        assert_eq!('X', item.symbol);
        assert_eq!(0, item.colour);
        assert_eq!(1, item.weight);
        assert_eq!(1, item.value);
    }
}