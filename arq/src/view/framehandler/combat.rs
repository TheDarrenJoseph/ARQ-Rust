use tui::Frame;
use tui::layout::{Alignment, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Paragraph};
use crate::character::battle::Battle;
use crate::character::equipment::{Equipment, EquipmentSlot, WeaponSlot};

use crate::engine::combat::CombatTurnChoice;
use crate::engine::level::Level;
use crate::map::map_view_areas::MapViewAreas;
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::option_list_selection::{MappedOption, OptionListSelection};

use crate::view::framehandler::{FrameData, FrameHandler};

use crate::widget::stateful::map_widget::MapWidget;

pub struct CombatFrameHandler {
    pub(crate) areas : Option<CombatViewAreas>,
    pub selection: OptionListSelection<CombatTurnChoice>,
    pub level: Level
}

pub struct CombatViewAreas {
    pub main_area : Area,
    pub console_area : Area,
    pub minimap_area : Area
}

pub struct ConsoleWidgets<'a> {
    paragraphs : Vec<(Paragraph<'a>, Rect)>,
    window : (Block<'a>, Rect)
}

impl CombatFrameHandler {
    pub fn new(level: Level) -> CombatFrameHandler {
        CombatFrameHandler { areas: None, selection: OptionListSelection::new(), level }
    }

    fn build_options(&self, equipment: Equipment) -> Vec<MappedOption<CombatTurnChoice>> {
        let mut choices = Vec::new();

        let slots = equipment.get_slots();
        if slots.contains_key(&EquipmentSlot::PRIMARY) {
            choices.push(MappedOption { mapped: CombatTurnChoice::ATTACK(WeaponSlot::PRIMARY), name: String::from("Attack (Primary)"), size: 16});
        }

        if slots.contains_key(&EquipmentSlot::SECONDARY) {
            choices.push(MappedOption {  mapped: CombatTurnChoice::ATTACK(WeaponSlot::SECONDARY), name: String::from("Attack (Secondary)"), size: 18});
        }

        choices.push(MappedOption { mapped: CombatTurnChoice::FLEE, name: String::from("Flee"), size: 4});

        return choices;
    }

    fn build_console_widgets(&self) -> ConsoleWidgets {
        let console_area = self.areas.as_ref().unwrap().console_area;
        let console_window_block = Block::default()
            .borders(Borders::ALL);

        let highlighted_option = self.selection.index;
        let mut i = 0;

        let mut paragraphs: Vec<(Paragraph, Rect)> = Vec::new();
        let largest_option_length = self.selection.options.iter().max_by_key(|o| o.size).unwrap().size.clone() as u16;
        for option in &self.selection.options {
            let mut style = Style::default();
            if i == highlighted_option {
                style = style.add_modifier(Modifier::REVERSED);
            }

            let paragraph = Paragraph::new(Text::from(option.name.clone()))
                .style(style)
                .alignment(Alignment::Left);

            let offset_y = i + 1;
            let text_area = Rect::new(console_area.start_position.x.clone() + 1, console_area.start_position.y.clone() + offset_y, largest_option_length, 1);

            paragraphs.push((paragraph, text_area));
            i += 1;
        }

        return  ConsoleWidgets { window: (console_window_block, console_area.to_rect()), paragraphs };
    }
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
        let battle = data.data;
        let mut characters = battle.characters;
        let player = characters.get_player_mut().unwrap();
        let player_equipment = player.get_equipment_mut().clone();

        if self.selection.options.len() == 0 {
            self.selection = OptionListSelection { options: self.build_options(player_equipment.clone()), index: 0 };
        }

        let player_name = player.get_name().clone();
        let _player_equipment_slots = player_equipment.get_slots();

        let enemy = characters.get_npcs_mut().first_mut().unwrap();
        let enemy_equipment = enemy.get_equipment_mut().clone();
        let enemy_name = enemy.get_name();

        let main_area = self.areas.as_ref().unwrap().main_area;

        let title = String::from(format!("{:─^width$}", "COMBAT───", width = main_area.width as usize));
        let title_span = Span::from(title);

        let main_window_block = Block::default()
            .title(title_span)
            .borders(Borders::ALL);
        frame.render_widget(main_window_block, main_area.to_rect());

        // Split the main window into 2 columns / sides
        let side_width = (main_area.width - 2) / 2;
        let side_height= main_area.height - 2;

        // Start inside the border (+1)
        let left_side_start_position = Position { x: main_area.start_position.x + 1, y: main_area.start_position.y + 1 };
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

        let right_side_start_position = Position { x: left_side_area.end_position.x, y: main_area.start_position.y + 1 };
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

        // Build widget / area tuples
        let console_widgets = self.build_console_widgets();
        let console_block = console_widgets.window.0;
        let console_area = console_widgets.window.1;
        frame.render_widget(console_block, console_area);
        // Unpack these tuples and render them
        for paragraph_area in console_widgets.paragraphs {
            frame.render_widget(paragraph_area.0, paragraph_area.1);
        }

        let minimap_display_area = self.areas.as_ref().unwrap().minimap_area.to_rect();
        let mut player_global_pos = self.level.characters.get_player_mut().unwrap().get_global_position();


        // Offset the the player pos by half of the minimap size to center it
        let half_minimap_width = ( minimap_display_area.width / 2) as i32;
        let half_minimap_height = ( minimap_display_area.height / 2) as i32;
        let minimap_target_pos = player_global_pos.offset(-half_minimap_width, -half_minimap_height);
        let minimap_target_area = Area::new(minimap_target_pos, minimap_display_area.width,minimap_display_area.height);

        let map_area = self.level.map.as_ref().unwrap().area.clone();
        let map_view_area = Area::from_rect(minimap_display_area);
        let map_display_area = minimap_target_area;
        let map_view_areas = MapViewAreas { map_area, map_view_area, map_display_area };
        let map_widget = MapWidget::new(map_view_areas);

        let dummy_area = Area::new(Position::new(0,0),0,0);
        frame.render_stateful_widget(map_widget, dummy_area.to_rect(), &mut self.level);
    }
}