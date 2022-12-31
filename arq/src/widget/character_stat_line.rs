use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{StatefulWidget, Widget};

use crate::character::character_details::CharacterDetails;
use crate::widget::{StatefulWidgetState, StatefulWidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct CharacterStatLineWidget {
    health: i8,
    loot_score: i32,
    character_details : CharacterDetails
}

impl CharacterStatLineWidget {
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

pub fn build_character_stat_line(health: i8, character_details: CharacterDetails, loot_score: i32) -> CharacterStatLineWidget {
    CharacterStatLineWidget { health, loot_score, character_details }
}



impl Widget for CharacterStatLineWidget {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let health_header = String::from("Health: ");
        let health_value = self.health.to_string();
        buf.set_string(area.x , area.y, health_header.as_str(), Style::default().fg(Color::Green));
        buf.set_string(area.x + health_header.len() as u16, area.y,self.health.to_string(), Style::default());

        let loot_offset = area.x + 1 + health_header.len() as u16 + health_value.len() as u16 + 2;
        let loot_header = String::from("Loot: ");
        let loot_value = self.loot_score.to_string();
        let loot_text =  format!("{:0>6}", loot_value);
        buf.set_string(loot_offset, area.y, loot_header.as_str(), Style::default().fg(Color::Blue));
        buf.set_string(loot_offset + loot_header.len() as u16 + 1, area.y, loot_text, Style::default());
    }
}