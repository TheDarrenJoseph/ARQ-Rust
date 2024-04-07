use std::ptr::eq;
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;
use crate::character::equipment::{all_equipment_slots, Equipment};
use crate::ui::ui_areas::UI_AREA_NAME_MAIN;
use crate::view::framehandler::{FrameData, FrameHandler};

pub struct CharacterEquipmentFrameHandler {
}

impl <B : tui::backend::Backend> FrameHandler<B, Equipment> for CharacterEquipmentFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<Equipment>) {

        // TODO potentially use ui_areas in future?
        //let main_area = data.get_ui_areas().get_area(UI_AREA_NAME_MAIN).expect("Main UIArea should be available.");
        let frame_area = data.frame_area;

        let title_lengths : Vec<usize> = all_equipment_slots().iter().map(|s| s.to_string().len()).collect();
        // +1 to ensure good formatting
        let max_title_length : usize = *title_lengths.iter().max().unwrap() + 1;

        // Part one, render the left-hand side titles for each slot
        let mut title_spans_list = Vec::new();
        for slot in all_equipment_slots() {
            let title = Span::from(slot.to_string());
            let spans = Spans::from(title);
            title_spans_list.push(spans);
        }

        let paragraph = Paragraph::new(title_spans_list)
            .style(Style::default())
            .alignment(tui::layout::Alignment::Left);
        frame.render_widget(paragraph, frame_area.to_rect());

        // Part two, render the equipment names offset to the right
        let equipment = data.data;
        let mut name_spans_list = Vec::new();
        for slot in all_equipment_slots() {
            let name = if let Some(equipped_item) = equipment.get_item(slot) {
                Span::from(equipped_item.get_name())
            } else {
                Span::from("Empty")
            };
            let spans = Spans::from(name);
            name_spans_list.push(spans);
        }

        // Render a paragraph for the equipment names
        let mut names_area = frame_area.to_rect();
        names_area.x = names_area.x + max_title_length as u16;
        names_area.width -= max_title_length as u16;
        let paragraph = Paragraph::new(name_spans_list)
            .style(Style::default())
            .alignment(tui::layout::Alignment::Left);
        frame.render_widget(paragraph, names_area);
    }
}
