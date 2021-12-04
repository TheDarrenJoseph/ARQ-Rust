use crate::items::{Item, ItemType};
use uuid::Uuid;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum ContainerType {
    ITEM, // No storage, just a wrapped Item
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

    fn get_weight_total(&self) -> i32 {
        let mut weight_total = 0;
        match self.container_type {
            ContainerType::OBJECT | ContainerType::AREA => {
                weight_total += self.get_self_item().weight.clone();
                for c in self.get_contents() {
                    weight_total += c.get_weight_total();
                }
                return weight_total;
            },
            _ => {
                weight_total += self.get_self_item().weight.clone();
            }
        }
        weight_total
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
               let total_weight = self.get_weight_total();
               let adding_weight_limit = container.weight_limit;
               let max_weight_limit = total_weight + adding_weight_limit;

               let within_potential_weight_limit = max_weight_limit <= self.weight_limit;
               let potential_weight = total_weight.clone() + container.get_weight_total();
               if within_potential_weight_limit && potential_weight <= self.weight_limit {
                   self.contents.push(container);
               }
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
                if self.get_weight_total() + item.weight <= self.weight_limit {
                    let container = wrap_item(item.clone());
                    self.contents.push(container);
                }
            }
        }
    }

}

pub fn wrap_item(item: Item) -> Container {
    Container { item, container_type: ContainerType::ITEM, weight_limit: 0, contents: Vec::new() }
}

pub fn build(id: Uuid, name: String, symbol: char, weight : i32, value : i32, container_type : ContainerType, weight_limit : i32) -> Container {
    let container_item = crate::items::build_container_item(id, name, symbol, weight, value);
    Container { item: container_item, container_type, weight_limit, contents: Vec::new()}
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::container::ContainerType;

    #[test]
    fn test_container_build() {
        let id = Uuid::new_v4();
        let container =  crate::container::build(id, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        assert_eq!(id, container.item.get_id());
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
    fn test_container_add_item() {
        // GIVEN we have a valid container
        let mut container =  crate::container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we call to add a new item
        let item = crate::items::build_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 1, 1);
        container.add_item(item);

        // THEN we expect it's contents size to increase
        assert_eq!(1, container.get_contents().len());
    }

    #[test]
    fn test_container_add_item_weight_limit() {
        // GIVEN we have a valid container
        let mut container =  crate::container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we try to add more items than the supported weight limit
        let item = crate::items::build_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 100, 1);
        let item2 = crate::items::build_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 1, 1);
        container.add_item(item);
        container.add_item(item2);

        // THEN we expect only the first item to be added
        assert_eq!(1, container.get_contents().len());
    }

    #[test]
    fn test_container_add() {
        // GIVEN we have a valid container
        let mut container =  crate::container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 0, 1,  ContainerType::OBJECT, 40);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we call to add either an wrapped ITEM or OBJECT container
        let gold_bar = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let bag_object = crate::container::build(Uuid::new_v4(), "Bag".to_owned(), 'X', 0, 1, ContainerType::OBJECT, 30);
        container.add(gold_bar);
        container.add(bag_object);

        // THEN we expect it's contents size to increase
        assert_eq!(2, container.get_contents().len());
    }

    #[test]
    fn test_container_add_weight_limit() {
        // GIVEN we have a valid container with a weight limit of 40
        let mut container =  crate::container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 0, 1,  ContainerType::OBJECT, 40);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        let gold_bar_1 = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let mut bag_object = crate::container::build(Uuid::new_v4(), "Bag".to_owned(), 'X', 0, 1, ContainerType::OBJECT, 30);
        let gold_bar_2 = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let gold_bar_3 = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let gold_bar_4 = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let lockpick_1 = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Lockpick".to_owned(), 'X', 1, 5));
        let lockpick_2 = crate::container::wrap_item(crate::items::build_item(Uuid::new_v4(), "Lockpick".to_owned(), 'X', 1, 5));
        bag_object.add(gold_bar_2);
        bag_object.add(gold_bar_3);
        bag_object.add(gold_bar_4);

        // WHEN we add more items than the container can support (total of 42 weight)
        container.add(gold_bar_1);
        container.add(bag_object);
        container.add(lockpick_1);
        container.add(lockpick_2);

        // THEN we expect only the first 2 objects to be added
        // Along with the bag contents
        assert_eq!(2, container.get_contents().len());
        assert_eq!(3, container.get_contents()[1].get_contents().len());
    }

    #[test]
    fn test_get_loot_value_empty() {
        // GIVEN we have a valid container with no items
        let mut container =  crate::container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // WHEN we call to get the total item value
        let total_value = container.get_loot_value();
        // THEN we expect 0 to be returned
        assert_eq!(0, total_value);
    }

    #[test]
    fn test_get_loot_value() {
        // GIVEN we have a valid container
        let mut container =  crate::container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());

        // AND we've added 2 items with different values
        let gold_bar = crate::items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);

        let silver_bar = crate::items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);
        assert_eq!(2, container.get_contents().len());

        // WHEN we call to get their total value
        let total_value = container.get_loot_value();
        // THEN we expect the total of all the item values to be returned
        assert_eq!(150, total_value);
    }
}