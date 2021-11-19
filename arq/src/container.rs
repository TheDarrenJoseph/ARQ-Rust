use crate::items::{Item, ItemType};

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum ContainerType {
    ITEM, // No inventory, just item set
    OBJECT, // Movable container i.e Bags
    AREA // Fixed container i.e Floor, Chests, Player's inventory
}

/*
* Container is an item wrapper at it's most basic (ITEM),
* otherwise a container for storing items (i.e: OBJECT, AREA) which are also Container instances
*/
#[derive(Clone, Debug)]
pub struct Container {
    item : Item,
    pub container_type : ContainerType,
    weight_limit : i32,
    contents : Vec<Container>
}

impl Container {
    pub fn get_self_item(&self) -> &Item {
        &self.item
    }

    pub fn get_contents(&self) -> &Vec<Container> {
        &self.contents
    }

    pub fn get(&self, index: i32) -> &Container {
        &self.contents[index as usize]
    }

    pub fn get_mut(&mut self, index: i32) -> &mut Container {
        &mut self.contents[index as usize]
    }

    pub fn get_loot_value(&self) -> i32 {
        let mut loot_total = 0;
        for c in &self.contents {
            match c.container_type {
                ContainerType::ITEM => {
                    loot_total += c.get_self_item().value;
                },
                ContainerType::OBJECT | ContainerType::AREA => {
                    loot_total += c.get_self_item().value;
                    for c in c.get_contents() {
                        loot_total += c.get_loot_value();
                    }
                }
            }

        }
        loot_total
    }

    pub fn add(&mut self, container : Container) {
        match container.container_type {
           ContainerType::ITEM | ContainerType::OBJECT => {
                self.contents.push(container);
            },
            _ => {}
        }

    }

    pub fn can_open(&self) -> bool {
        self.container_type ==  ContainerType::OBJECT || self.container_type ==  ContainerType::AREA
    }

    pub fn add_item(&mut self, item : Item) {
        match item.item_type {
            // Container items should only ever be the meta item for a container
            ItemType::CONTAINER => {
                return;
            },
            _ => {
                let container = Container { item: item.clone(), container_type: ContainerType::ITEM, weight_limit: item.weight.clone(), contents : Vec::new() };
                self.contents.push(container);
            }
        }
    }
}

pub fn build(id: u64, name: String, symbol: char, weight : i32, value : i32, container_type : ContainerType, weight_limit : i32) -> Container {
    let container_item = crate::items::build_container_item(id, name, symbol, weight, value);
    Container { item: container_item, container_type, weight_limit, contents: Vec::new()}
}

#[cfg(test)]
mod tests {
    use crate::container::ContainerType;

    #[test]
    fn test_container_build() {
        let container =  crate::container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        assert_eq!(0, container.item.get_id());
        assert_eq!("Test Container", container.item.name);
        assert_eq!('X', container.item.symbol);
        assert_eq!(0, container.item.colour);
        assert_eq!(1, container.item.weight);
        assert_eq!(1, container.item.value);

        assert_eq!(ContainerType::OBJECT, container.container_type);
        assert_eq!(100, container.weight_limit);

        let contents = container.get_contents();
        assert_eq!(0, contents.len());
    }

    #[test]
    fn test_container_add() {
        // GIVEN we have a valid container
        let mut container =  crate::container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we call to add a new item
        let item = crate::items::build_item(1, "Test Item".to_owned(), 'X', 1, 1);
        container.add_item(item);

        // THEN we expect it's contents size to increase
        assert_eq!(1, container.get_contents().len());
    }

    #[test]
    fn test_get_loot_value_empty() {
        // GIVEN we have a valid container with no items
        let mut container =  crate::container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // WHEN we call to get the total item value
        let total_value = container.get_loot_value();
        // THEN we expect 0 to be returned
        assert_eq!(0, total_value);
    }

    #[test]
    fn test_get_loot_value() {
        // GIVEN we have a valid container
        let mut container =  crate::container::build(0, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());

        // AND we've added 2 items with different values
        let gold_bar = crate::items::build_item(1, "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);

        let silver_bar = crate::items::build_item(1, "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);
        assert_eq!(2, container.get_contents().len());

        // WHEN we call to get their total value
        let total_value = container.get_loot_value();
        // THEN we expect the total of all the item values to be returned
        assert_eq!(150, total_value);
    }
}