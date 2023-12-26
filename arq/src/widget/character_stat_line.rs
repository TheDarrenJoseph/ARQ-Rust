use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Widget};

use crate::character::character_details::CharacterDetails;


#[derive(Clone)]
#[derive(Debug)]
pub struct CharacterStatLineWidget {
    level: i32,
    health: i8,
    loot_score: i32,
    character_details : CharacterDetails
}

impl CharacterStatLineWidget {
    pub fn new(level: i32, health: i8, character_details: CharacterDetails, loot_score: i32) -> CharacterStatLineWidget {
        CharacterStatLineWidget { level, health, loot_score, character_details }
    }

    pub fn set_level(&mut self, level: i32) {
        self.level = level;
    }

    pub fn get_health(&self) -> i8 {
        self.health
    }

    pub fn set_health(&mut self, health: i8) {
        self.health = health;
    }

    pub fn get_loot_score(&self) -> i32 {
        self.loot_score
    }

    pub fn set_loot_score(&mut self, loot_score: i32) {
        self.loot_score = loot_score;
    }

    pub fn get_character_details(&self) -> &CharacterDetails {
        &self.character_details
    }

    pub fn set_character_details(&mut self, character_details: CharacterDetails) {
        self.character_details = character_details;
    }
}

fn calculate_offset(x: u16, previous_header: String, previous_value: String) -> u16 {
    x + 1 + previous_header.len() as u16 + previous_value.len() as u16 + 2
}

impl Widget for CharacterStatLineWidget {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let level_header = String::from("Level: ");
        let level_text =  format!("{:0>3}",  self.level.to_string());
        buf.set_string(area.x , area.y, level_header.as_str(), Style::default().fg(Color::Yellow));
        buf.set_string(area.x + level_header.len() as u16, area.y,level_text.clone(), Style::default());

        let health_offset = calculate_offset(area.x, level_header, level_text);
        let health_header = String::from("Health: ");
        let loot_text =  format!("{:0>3}",  self.health.to_string());
        buf.set_string(health_offset , area.y, health_header.as_str(), Style::default().fg(Color::Green));
        buf.set_string(health_offset + health_header.len() as u16, area.y,loot_text.clone(), Style::default());

        let loot_offset = calculate_offset(health_offset, health_header, loot_text);
        let loot_header = String::from("Loot: ");
        let loot_text =  format!("{:0>6}",  self.loot_score.to_string());
        buf.set_string(loot_offset, area.y, loot_header.as_str(), Style::default().fg(Color::Blue));
        buf.set_string(loot_offset + loot_header.len() as u16 + 1, area.y, loot_text, Style::default());
    }
}