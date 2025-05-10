use std::convert::TryInto;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Modifier, StatefulWidget, Style};
use ratatui::widgets::{Block, Borders, Widget};
use termion::event::Key;
use crate::item_list_selection::{ItemListSelection, ListSelection};
use crate::map::objects::container::Container;
use crate::ui::event::Event;
use crate::ui::ui_areas::{UIAreas, UI_AREA_NAME_MAIN};
use crate::ui::ui_util::build_paragraph;
use crate::view::framehandler::util::paging::{build_page_count, build_weight_limit};
use crate::view::framehandler::util::tabling::{build_headings, Column};
use crate::view::model::usage_line::UsageLine;

#[derive(Debug, Clone)]
pub struct ContainerWidget {
    pub(crate) columns : Vec<Column>,
    pub(crate) row_count: i32,
}

pub struct ContainerWidgetData {
    pub container : Container,
    pub ui_areas: UIAreas,
    pub item_list_selection : ItemListSelection,
    pub(crate) usage_line : UsageLine
}

impl ContainerWidgetData {
    pub async fn handle_event(&mut self, event: Event) -> ContainerWidgetEventHandlingResult {
        match event {
            Event::Termion(termion_event) => {
                match termion_event {
                    termion::event::Event::Key(key) => {
                        match key {
                            Key::Up => {
                                self.item_list_selection.move_up();
                            },
                            Key::Down => {
                                self.item_list_selection.move_down();
                            },
                            Key::Char('\n') => {
                                self.item_list_selection.toggle_select();
                            },
                            Key::Esc => {
                                if self.item_list_selection.is_selecting() {
                                    self.item_list_selection.cancel_selection();
                                } else {
                                    return ContainerWidgetEventHandlingResult::Exit
                                }
                            }
                            _ => {}
                        }

                    },
                    _ => {}
                }
                ContainerWidgetEventHandlingResult::Continue
            },
            _ => {
                ContainerWidgetEventHandlingResult::Continue
            }
        }
    }
}

pub enum ContainerWidgetEventHandlingResult {
    Continue,
    Exit
}

impl StatefulWidget for ContainerWidget {
    type State = ContainerWidgetData;

    fn render(mut self, _: Rect, buf: &mut Buffer, data: &mut ContainerWidgetData) {
        let main_area = data.ui_areas.get_area(UI_AREA_NAME_MAIN).unwrap();
        let frame_size = main_area.area.to_rect();
        
        let container = &mut data.container;
        let item_list_selection = &mut data.item_list_selection;
        let usage_line = &mut data.usage_line;

        let window_block = Block::default()
            .borders(Borders::ALL)
            .title(container.get_self_item().get_name().clone());
        let window_area = Rect::new(frame_size.x.clone(), frame_size.y.clone(), frame_size.width.clone(), frame_size.height.clone());
        let inventory_item_lines = window_area.height - 3;
        self.row_count = inventory_item_lines as i32;
        item_list_selection.page_line_count = inventory_item_lines as i32;
        window_block.render(window_area, buf);

        let headings = build_headings(self.columns.clone());
        let headings_area = Rect::new(frame_size.x.clone() + 1, frame_size.y.clone() + 1, frame_size.width.clone() - 4, 2);
        headings.render(headings_area, buf);

        let mut line_index = 0;
        let start_index = item_list_selection.get_start_index();
        let end_of_page_representive_index = item_list_selection.get_end_of_page_index();

        if !container.get_contents().is_empty() {
            let view_contents = &container.get_contents()[start_index as usize..=end_of_page_representive_index as usize];
            for c in view_contents {
                let item_index = start_index.clone() + line_index.clone();
                let item = &c.get_self_item();
                // The x offset is the starting x 
                // + 1 to avoid the left-hand border
                let mut x_offset: u16 = frame_size.x.clone() + 1;
                // The y offset is the starting y 
                // + 2 (to avoid top border and the header row) 
                // + line index (to avoid previous lines)
                let y_offset: u16 = frame_size.y.clone() as u16 + 2 + line_index.clone() as u16;

                let current_index = item_list_selection.is_focused(item_index);
                let selected = item_list_selection.is_selected(item_index);

                for column in &self.columns {
                    let text = crate::view::framehandler::container::build_column_text(column, item);
                    let mut column_text = build_paragraph(text);
                    if current_index.clone() && selected.clone() {
                        column_text = column_text.style(Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED));
                    } else if current_index {
                        column_text = column_text.style(Style::default().add_modifier(Modifier::REVERSED));
                    } else if selected {
                        column_text = column_text.style(Style::default().fg(Color::Green));
                    }

                    let column_length = column.size as i8;
                    let text_area = Rect::new(x_offset.clone(), y_offset.clone(), column_length.try_into().unwrap(), 1);
                    column_text.render(text_area, buf);
                    x_offset += column_length as u16;
                }
                line_index += 1;
            }
            
            let usage_description = usage_line.describe();
            let usage_text = build_paragraph(usage_description.clone());
            let text_area = Rect::new(window_area.x.clone() + 1, window_area.y.clone() + window_area.height.clone() - 1, usage_description.len().try_into().unwrap(), 1);
            usage_text.render(text_area, buf);

            // From right hand to left hand side draw the info text
            let page_count = build_page_count(&item_list_selection, window_area.clone());
            page_count.0.render(page_count.1, buf);

            let page_count_text_length = page_count.2;
            let weight_limit = build_weight_limit(&data.container, window_area.clone(), page_count_text_length);
            weight_limit.0.render(weight_limit.1, buf);
        }
    }
}