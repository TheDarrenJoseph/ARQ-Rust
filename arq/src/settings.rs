pub struct Setting<T> {
    pub name : String,
    pub value : T
}

pub struct EnumSettings {
    pub settings : Vec<Setting<bool>>
}

pub trait Toggleable {
    fn toggle(&mut self);
}

impl Toggleable for Setting<bool> {
    fn toggle(&mut self) {
        self.value = !self.value;
    }
}