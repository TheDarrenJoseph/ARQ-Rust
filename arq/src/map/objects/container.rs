use crate::map::objects::items::{build_container_item, Item, ItemType};
use uuid::Uuid;
use std::convert::TryInto;
use std::fmt;
use std::slice::Iter;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum ContainerType {
    ITEM, // No storage, just a wrapped Item
    OBJECT, // Movable container i.e Bags
    AREA // Fixed container i.e Floor, Player's inventory
}

impl fmt::Display for ContainerType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
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
    pub fn is_true_container(&self) -> bool {
        let container_type_valid = (self.container_type == ContainerType::OBJECT) | (self.container_type == ContainerType::AREA);
        let item_type_valid = self.item.item_type == ItemType::CONTAINER;
        return container_type_valid && item_type_valid;
    }

    pub fn get_self_item(&self) -> &Item {
        &self.item
    }

    pub fn id_equals(&self, other: &Container) -> bool {
        self.item.get_id() == other.get_self_item().get_id()
    }

    pub fn get_contents(&self) -> &Vec<Container> {
        &self.contents
    }

    pub fn get_contents_mut(&mut self) -> &mut Vec<Container> {
        &mut self.contents
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type.clone()
    }

    // Finds all subcontainers (OBJECTs) within this container's topmost level
    pub fn find_subcontainers(&mut self) -> Vec<&mut Container> {
        let mut containers = Vec::new();
        for c in &mut self.contents {
            if c.container_type == ContainerType::OBJECT {
                containers.push(c);
            }
        }
        containers
    }

    fn count_contents(&self) -> usize {
        let mut item_count = 0;
        for c in self.get_contents() {
            item_count += c.get_item_count();
        }
        return item_count;
    }

    // Returns the top-level content count if this is a 'true' container, 0 otherwise (i.e a wrapped item)
    pub fn get_content_count(&self) -> usize {
        return if self.is_true_container() {
            self.contents.len()
        } else {
            0
        }
    }

    // Returns the total count of all contents including itself
    pub fn get_item_count(&self) -> usize {
        return if self.is_true_container() {
            let mut item_count = 0;
            for c in self.get_contents() {
                item_count += 1; // +1 for self item
                if c.is_true_container() {
                    item_count += c.count_contents();
                }
            }
            item_count
        } else {
            return 1;
        }
    }

    pub fn get_weight_limit(&self) -> i32 {
        self.weight_limit.clone()
    }

    pub fn can_fit_container_item(&self, item: &Container) -> bool {
        let weight_limit = self.weight_limit.clone();
        let content_weight_total = self.get_contents_weight_total();
        let free_weight = weight_limit - content_weight_total;
        item.get_weight_total() <= free_weight
    }

    pub fn get(&self, index: i32) -> &Container {
        &self.contents[index as usize]
    }

    pub fn get_optional(&self, index: i32) -> Option<&Container> {
        self.contents.get(index as usize)
    }

    pub fn find(&self, item: &Item) -> Option<&Container> {
        // Initial pass
        let found = self.contents.iter().find(|c| {
            let expected_id = item.get_id();
            let self_item = c.get_self_item();
            self_item.get_id() == expected_id
        });

        if found.is_some() {
            found
        } else {
            // Recurse
            return self.contents.iter().flat_map(|c| {
                return c.find(item);
            }).next();
        }
    }

    pub fn push(&mut self, containers : Vec<Container>) {
        self.contents.extend(containers);
    }

    pub fn insert(&mut self, index: usize, containers : Vec<Container>) {
        let mut to_move : Vec<Container> = Vec::new();
        for c in containers {
            to_move.push(c.clone())
        }
        self.contents.splice(index..index, to_move);
    }

    pub fn replace_container(&mut self, new: Container) {
        if let Some(c) = self.contents.iter_mut().find(|c| c.id_equals(&new)) {
            *c = new;
        }
    }

    pub fn replace(&mut self, index: usize, container : Container) {
        if self.contents.len() > 0 && index < self.contents.len() {
            self.contents[index] = container;
        }
    }

    pub fn remove_item(&mut self, item : &Container) -> bool {
        if let Some(position) = self.position(item) {
            self.contents.remove(position);
            return true;
        }
        return false;
    }

    pub fn remove_matching_items(&mut self, items : Vec<Container>) -> bool {
        let mut removed = Vec::new();
        for item in items.iter() {
            let mut success = self.remove_item(item);
            removed.push(item.clone());
            if !success {
                for c in self.contents.iter_mut() {
                    if c.remove_matching_items(items.clone()) {
                        removed = items.clone();
                    }
                }
            }
        }
        return removed.len() == items.len();
    }

    pub fn remove_items(&mut self, items : Vec<&Container>) {
        for item in items.iter() {
            if let Some(position) = self.position(item) {
                self.contents.remove(position);
            }
        }
    }

    pub fn position(&self, container: &Container) -> Option<usize> {
        self.contents.iter().position(|c| {
            let expected_id = container.get_self_item().get_id();
            let self_item = c.get_self_item();
            self_item.get_id() == expected_id
        })
    }

    pub fn item_position(&self, item: &Item) -> Option<usize> {
        self.contents.iter().position(|c| {
            let expected_id = item.get_id();
            let self_item = c.get_self_item();
            self_item.get_id() == expected_id
        })
    }

    pub fn find_mut(&mut self, item: &Item) -> Option<&mut Container> {
        let expected_id = item.get_id();
        for c in self.contents.iter_mut() {
            if c.get_self_item().get_id() == expected_id {
                return Some(c)
            } else {
                if let Some(subcontainer) = c.find_mut(item) {
                    return Some(subcontainer);
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, index: i32) -> Option<&mut Container> {
        let contents_size : i32 = self.contents.len().try_into().unwrap();
        if index >= 0 && index < contents_size {
            return Some(&mut self.contents[index as usize])
        }
        None
    }

    pub(crate) fn get_weight_total(&self) -> i32 {
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

    pub(crate) fn get_contents_weight_total(&self) -> i32 {
        return  self.get_weight_total() - self.get_self_item().weight.clone();
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

    // Adds a container to this one only if this is a true container and the adding Container is a storable/movable type (ITEM or OBJECT)
    pub fn add(&mut self, container : Container) {
        if self.is_true_container() {
            match container.container_type {
                ContainerType::ITEM | ContainerType::OBJECT => {
                    let total_weight = self.get_contents_weight_total();
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
    }

    pub fn can_open(&self) -> bool {
        // TODO use is_true_container?
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

    pub fn add_items(&mut self, items : Vec<Item>) {
        for i in items {
            self.add_item(i);
        }
    }

    pub fn to_cloned_item_list(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for c in self.get_contents() {
            items.push(c.get_self_item().clone());
        }
        items
    }
}

pub fn wrap_item(item: Item) -> Container {
    Container { item, container_type: ContainerType::ITEM, weight_limit: 0, contents: Vec::new() }
}

pub fn build(id: Uuid, name: String, symbol: char, weight : i32, value : i32, container_type : ContainerType, weight_limit : i32) -> Container {
    let container_item = build_container_item(id, name, symbol, weight, value);
    Container { item: container_item, container_type, weight_limit, contents: Vec::new()}
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::map::objects::container;
    use crate::map::objects::container::{build, ContainerType};
    use crate::map::objects::items;
    use crate::map::tile::Colour;

    #[test]
    fn test_container_build() {
        let id = Uuid::new_v4();
        let container =  build(id, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);

        assert_eq!(id, container.item.get_id());
        assert_eq!("Test Container", container.item.name);
        assert_eq!('X', container.item.symbol);
        assert_eq!(Colour::White, container.item.colour);
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
        let mut container =  build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we call to add a new item
        let item = items::build_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 1, 1);
        container.add_item(item);

        // THEN we expect it's contents size to increase
        assert_eq!(1, container.get_contents().len());
    }

    #[test]
    fn test_container_add_item_weight_limit() {
        // GIVEN we have a valid container
        let mut container =  build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we try to add more items than the supported weight limit
        let item = items::build_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 100, 1);
        let item2 = items::build_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 1, 1);
        container.add_item(item);
        container.add_item(item2);

        // THEN we expect only the first item to be added
        assert_eq!(1, container.get_contents().len());
    }

    #[test]
    fn test_container_add_item_container_item() {
        // GIVEN we have a valid container
        let mut container =  build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we call to add a new item that's of CONTAINER type
        let item = items::build_container_item(Uuid::new_v4(), "Test Item".to_owned(), 'X', 1, 1);
        container.add_item(item);

        // THEN we expect nothing to happen as it's an unsupported type
        assert_eq!(0, container.get_contents().len());
    }

    #[test]
    fn test_container_add() {
        // GIVEN we have a valid container
        let mut container = build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 0, 1,  ContainerType::OBJECT, 40);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we call to add either an wrapped ITEM or OBJECT container
        let gold_bar = container::wrap_item(items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let bag_object = build(Uuid::new_v4(), "Bag".to_owned(), 'X', 0, 1, ContainerType::OBJECT, 30);
        container.add(gold_bar);
        container.add(bag_object);

        // THEN we expect it's contents size to increase
        assert_eq!(2, container.get_contents().len());
    }

    #[test]
    fn test_container_add_weight_limit() {
        // GIVEN we have a valid container with a weight limit of 40
        let mut container =  build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 0, 1,  ContainerType::OBJECT, 40);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        let gold_bar_1 = container::wrap_item(items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let mut bag_object = container::build(Uuid::new_v4(), "Bag".to_owned(), 'X', 0, 1, ContainerType::OBJECT, 30);
        let gold_bar_2 = container::wrap_item(items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let gold_bar_3 = container::wrap_item(items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let gold_bar_4 = container::wrap_item(items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 10, 100));
        let lockpick_1 = container::wrap_item(items::build_item(Uuid::new_v4(), "Lockpick".to_owned(), 'X', 1, 5));
        let lockpick_2 = container::wrap_item(items::build_item(Uuid::new_v4(), "Lockpick".to_owned(), 'X', 1, 5));
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
    fn test_container_add_area_item() {
        // GIVEN we have a valid container
        let mut container =  build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it has no items in it's contents
        assert_eq!(0, container.get_contents().len());

        // WHEN we try to add an AREA container (immovable)
        let mut floor =  build(Uuid::new_v4(), "Floor".to_owned(), 'X', 1, 1, ContainerType::AREA, 100);
        container.add(floor);

        // THEN we expect nothing to happen as it's an unsupported type
        assert_eq!(0, container.get_contents().len());
    }

    #[test]
    fn test_get_loot_value_empty() {
        // GIVEN we have a valid container with no items
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // WHEN we call to get the total item value
        let total_value = container.get_loot_value();
        // THEN we expect 0 to be returned
        assert_eq!(0, total_value);
    }

    #[test]
    fn test_get_loot_value() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());

        // AND we've added 2 items with different values
        let gold_bar = items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);

        let silver_bar = items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);
        assert_eq!(2, container.get_contents().len());

        // WHEN we call to get their total value
        let total_value = container.get_loot_value();
        // THEN we expect the total of all the item values to be returned
        assert_eq!(150, total_value);
    }

    #[test]
    fn test_get_content_count() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // AND we've added 2 items with different values
        let gold_bar = items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);
        let silver_bar = items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);
        assert_eq!(2, container.get_contents().len());

        // WHEN we call to get the content count
        let count = container.get_content_count();
        // THEN we expect the total of the top-level contents to return
        assert_eq!(2, count);
    }

    #[test]
    fn test_get_content_count_nested() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // AND we've added 2 items with different values
        let gold_bar = items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);
        let silver_bar = items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);

        // AND we've added a container with it's own series of items
        let mut bag =  container::build(Uuid::new_v4(), "Bag".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 20);
        let coin1 = items::build_item(Uuid::new_v4(), "Gold Coin 1".to_owned(), 'X', 1, 10);
        let coin2 = items::build_item(Uuid::new_v4(), "Gold Coin 2".to_owned(), 'X', 1, 10);
        let coin3 = items::build_item(Uuid::new_v4(), "Gold Coin 3".to_owned(), 'X', 1, 10);
        bag.add_items(vec![coin1, coin2, coin3]);
        assert_eq!(3, bag.get_contents().len());
        container.add(bag);
        assert_eq!(3, container.get_contents().len());

        // WHEN we call to get the content count
        let count = container.get_content_count();
        // THEN we expect only the total of the top-level contents to return
        assert_eq!(3, count);
    }

    #[test]
    fn test_get_content_count_empty() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it contains nothing
        assert_eq!(0, container.get_contents().len());
        // WHEN we call to get the content count
        let count = container.get_content_count();
        // THEN
        assert_eq!(0, count);
    }

    #[test]
    fn test_get_item_count() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // AND we've added 2 items with different values
        let gold_bar = items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);
        let silver_bar = items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);
        assert_eq!(2, container.get_contents().len());

        // WHEN we call to get the item count
        let count = container.get_item_count();
        // THEN we expect the total count of all items to return
        assert_eq!(2, count);
    }

    #[test]
    fn test_get_item_count_nested() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        assert_eq!(0, container.get_contents().len());
        // AND we've added 2 items with different values
        let gold_bar = items::build_item(Uuid::new_v4(), "Gold Bar".to_owned(), 'X', 1, 100);
        container.add_item(gold_bar);
        let silver_bar = items::build_item(Uuid::new_v4(), "Silver Bar".to_owned(), 'X', 1, 50);
        container.add_item(silver_bar);

        // AND we've added a container with it's own series of items
        let mut bag =  container::build(Uuid::new_v4(), "Bag".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 20);
        let coin1 = items::build_item(Uuid::new_v4(), "Gold Coin 1".to_owned(), 'X', 1, 10);
        let coin2 = items::build_item(Uuid::new_v4(), "Gold Coin 2".to_owned(), 'X', 1, 10);
        let coin3 = items::build_item(Uuid::new_v4(), "Gold Coin 3".to_owned(), 'X', 1, 10);
        bag.add_items(vec![coin1, coin2, coin3]);
        assert_eq!(3, bag.get_contents().len());
        container.add(bag);
        assert_eq!(3, container.get_contents().len());

        // WHEN we call to get the item count
        let count = container.get_item_count();
        // THEN we expect the total count of all items to return
        assert_eq!(6, count);
    }

    #[test]
    fn test_get_item_count_empty() {
        // GIVEN we have a valid container
        let mut container =  container::build(Uuid::new_v4(), "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        // AND it contains nothing
        assert_eq!(0, container.get_contents().len());
        // WHEN we call to get the item count
        let count = container.get_item_count();
        // THEN
        assert_eq!(0, count);
    }
}