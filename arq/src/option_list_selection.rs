

pub struct MappedOption<T> {
    pub mapped : T,
    pub name : String,
    pub size : i8
}

// Selection state for a series of text / Column options
pub struct OptionListSelection<T> {
    pub options: Vec<MappedOption<T>>,
    pub index: u16
}

impl<T> OptionListSelection<T> {
    pub fn new() -> OptionListSelection<T> {
        OptionListSelection {
            options: vec![],
            index: 0
        }
    }
}