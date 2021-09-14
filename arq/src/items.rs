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