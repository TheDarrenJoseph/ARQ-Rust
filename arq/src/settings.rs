pub struct Setting<T> {
    pub name : String,
    pub value : T
}

pub struct Settings {
    pub bool_settings : Vec<Setting<bool>>,
    pub string_settings : Vec<Setting<String>>
}

impl Settings {
    pub fn find_string_setting_value(&self, name : String) -> Option<String> {
        let setting = self.string_settings.iter().find(|x| x.name == name);
        if let Some(s) = setting {
            return Some(s.value.clone());
        }
        None
    }

    pub fn find_bool_setting_value(&self, name : String) -> Option<bool> {
        let setting = self.bool_settings.iter().find(|x| x.name == name);
        if let Some(s) = setting {
            return Some(s.value.clone());
        }
        None
    }
}

pub trait Toggleable {
    fn toggle(&mut self);
}

impl Toggleable for Setting<bool> {
    fn toggle(&mut self) {
        self.value = !self.value;
    }
}