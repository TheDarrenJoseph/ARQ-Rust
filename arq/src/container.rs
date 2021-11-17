use crate::items::Item;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum ContainerType {
    OBJECT,
    AREA
}

#[derive(Clone, Debug)]
pub struct Container {
    item : Item,
    container_type : ContainerType,
    weight_limit : i32,
    contents : Vec<Item>
}

impl Container {
    pub fn get_contents(&self) -> &Vec<Item> {
        &self.contents
    }
    pub fn get_loot_value(&self) -> i32 {
        let mut loot_total = 0;
        for item in &self.contents {
         loot_total += item.value;
        }
        loot_total
    }

    pub fn add_item(&mut self, item : Item) {
        self.contents.push(item);
    }
}

pub fn build(id: u64, name: String, symbol: char, weight : i32, value : i32, container_type : ContainerType, weight_limit : i32) -> Container {
    let container_item = crate::items::build_container(id, name, symbol, weight, value);
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