use crate::map::Map;
use crate::character::Character;

pub struct Level {
    pub map : Option<Map>,
    pub characters : Vec<Character>
}