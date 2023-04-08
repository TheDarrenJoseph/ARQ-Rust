use tui::Frame;
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph};
use crate::character::battle::Battle;
use crate::character::equipment::Equipment;
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::ui::ui_areas::UIAreas;
use crate::view::framehandler::{FrameData, FrameHandler};

pub struct CombatFrameHandler {
    pub areas : Option<UIAreas>
}

fn list_equipment(equipment: Equipment) -> Paragraph<'static> {
    let mut spans = vec![];
    for slot in equipment.get_slots() {
        let item = slot.1;
        spans.push(Spans::from(item.get_name().clone()))
    }
    Paragraph::new(spans)
}

impl <B : tui::backend::Backend> FrameHandler<B, Battle> for CombatFrameHandler {
    fn handle_frame(&mut self, frame: &mut Frame<B>, data: FrameData<Battle>) {
        let mut battle = data.data;
        let mut characters = battle.characters;
        let player = characters.get_player_mut().unwrap();
        let player_equipment = player.get_equipment_mut().clone();
        let player_name = player.get_name().clone();
        let player_equipment_slots = player_equipment.get_slots();

        let enemy = characters.get_npcs_mut().first_mut().unwrap();
        let enemy_equipment = enemy.get_equipment_mut().clone();
        let enemy_name = enemy.get_name();

        let main_area = self.areas.as_ref().unwrap().get_main_area();

        let title = String::from(format!("{:─^width$}", "COMBAT───", width = main_area.width as usize));
        let title_span = Span::from(title);

        let main_window_block = Block::default()
            .title(title_span)
            .borders(Borders::ALL);
        frame.render_widget(main_window_block, main_area);

        // Split the main window into 2 columns / sides
        let side_width = (main_area.width - 2) / 2;
        let side_height= (main_area.height - 2);

        // Start inside the border (+1)
        let left_side_start_position = Position { x: main_area.x + 1, y: main_area.y + 1 };
        let left_side_area = build_rectangular_area(left_side_start_position, side_width, side_height);
        let left_side_block = Block::default()
            .title(Span::styled(player_name, Style::default().add_modifier(Modifier::UNDERLINED)))
            .borders(Borders::RIGHT);
        frame.render_widget(left_side_block, left_side_area.to_rect());

        // Player equipment area is the area within the left side window, adjust to fit
        let mut player_equipment_area = left_side_area.to_rect().clone();
        player_equipment_area.y += 1;
        player_equipment_area.height -= 1;
        let player_equipment_list = list_equipment(player_equipment);
        frame.render_widget(player_equipment_list, player_equipment_area);

        let right_side_start_position = Position { x: left_side_area.end_position.x, y: main_area.y + 1 };
        let right_side_area = build_rectangular_area(right_side_start_position, side_width, side_height);
        let right_side_block = Block::default()
            .title(Span::styled(enemy_name, Style::default().add_modifier(Modifier::UNDERLINED)))
            .borders(Borders::LEFT);
        frame.render_widget(right_side_block, right_side_area.to_rect());

        // Player equipment area is the area within the left side window, adjust to fit
        let mut enemy_equipment_area = right_side_area.to_rect().clone();
        enemy_equipment_area.x += 1;
        enemy_equipment_area.y += 1;
        enemy_equipment_area.height -= 1;
        let enemy_equipment_list = list_equipment(enemy_equipment);
        frame.render_widget(enemy_equipment_list, enemy_equipment_area);


        let console_area = self.areas.as_ref().unwrap().get_console_area().unwrap();
        let console_window_block = Block::default()
            .borders(Borders::ALL);
        frame.render_widget(console_window_block, console_area);
    }
}