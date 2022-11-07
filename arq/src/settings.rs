use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};

pub const SETTING_FOG_OF_WAR : &str = "Fog of War";
pub const SETTING_RNG_SEED : &str = "Map RNG Seed";
pub const SETTING_BG_MUSIC : &str = "Background music";

pub struct Setting<T> {
    pub name : String,
    pub value : T
}

pub struct Settings {
    pub bool_settings : Vec<Setting<bool>>,
    pub u32_settings : Vec<Setting<u32>>,
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

    pub fn find_u32_setting_value(&self, name : String) -> Option<u32> {
        let setting = self.u32_settings.iter().find(|x| x.name == name);
        if let Some(s) = setting {
            return Some(s.value.clone());
        }
        None
    }
}

pub fn build_settings() -> Settings {
    let fog_of_war : Setting<bool> = Setting { name: SETTING_FOG_OF_WAR.to_string(), value: false };

    let random_seed: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    let map_seed : Setting<String> = Setting { name: SETTING_RNG_SEED.to_string(), value: random_seed };
    let bg_music_volume : Setting<u32> = Setting { name: SETTING_BG_MUSIC.to_string(), value: 100 };
    Settings { bool_settings: vec![fog_of_war], string_settings: vec![map_seed], u32_settings: vec![bg_music_volume]}
}

pub trait Toggleable {
    fn toggle(&mut self);
}

impl Toggleable for Setting<bool> {
    fn toggle(&mut self) {
        self.value = !self.value;
    }
}
