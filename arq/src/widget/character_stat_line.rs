use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::StatefulWidget;

use crate::character::CharacterDetails;
use crate::widget::{Widget, WidgetType};

#[derive(Clone)]
#[derive(Debug)]
pub struct CharacterStatLineState {
    health: i8,
    loot_score: i32,
    character_details : CharacterDetails
}

pub fn build_character_stat_line(health: i8, character_details: CharacterDetails, loot_score: i32) -> Widget {
    let name_input_state = WidgetType::StatLine( CharacterStatLineState { health, character_details, loot_score});
    Widget{ state_type: name_input_state}
}


impl StatefulWidget for CharacterStatLineState {
    type State = CharacterStatLineState;

    fn render(self, _area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let health_header = String::from("Health: ");
        let health_value = self.health.to_string();
        buf.set_string(1, 0, health_header.as_str(), Style::default().fg(Color::Green));
        buf.set_string(health_header.len() as u16 + 1, 0, self.health.to_string(), Style::default());

        let loot_offset = health_header.len() as u16 + health_value.len() as u16 + 2;
        let loot_header = String::from("Loot: ");
        let loot_value = self.loot_score.to_string();
        let loot_text =  format!("{:0>6}", loot_value);
        buf.set_string(loot_offset, 0, loot_header.as_str(), Style::default().fg(Color::Blue));
        buf.set_string(loot_offset + loot_header.len() as u16 + 1, 0, loot_text, Style::default());
    }
}