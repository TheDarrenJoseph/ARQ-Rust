pub struct Item {
    id : u64,
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
}
pub fn build(id: u64, name: String, symbol: char, weight : i32, value : i32) -> Item {
    Item {id: id, name : name, symbol : symbol, colour: 0, weight: weight, value: value}
}

#[cfg(test)]
mod tests {
    use crate::container::ContainerType;

    #[test]
    fn test_build() {
        let item = crate::items::build(0, "Test Item".to_owned(), 'X', 1, 1);

        assert_eq!(0, item.get_id());
        assert_eq!("Test Item", item.name);
        assert_eq!('X', item.symbol);
        assert_eq!(0, item.colour);
        assert_eq!(1, item.weight);
        assert_eq!(1, item.value);
    }
}