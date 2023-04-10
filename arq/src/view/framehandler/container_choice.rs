use std::convert::TryInto;
use std::io::Error;
use termion::event::Key;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders};
use crate::item_list_selection::{build_list_selection, ItemListSelection, ListSelection};
use crate::map::objects::container::Container;

use crate::view::framehandler::util::paging::build_page_count;
use crate::view::framehandler::util::tabling::{build_headings, build_paragraph, Column};
use crate::view::{GenericInputResult, InputHandler, InputResult};
use crate::view::framehandler::{FrameData, FrameHandler};


#[derive(Clone)]
pub struct ContainerChoiceFrameHandler {
    choices: Vec<Container>,
    columns : Vec<Column>,
    pub item_list_selection : ItemListSelection,
}

pub fn build(choices: Vec<Container>) -> ContainerChoiceFrameHandler {
    let mut items = Vec::new();
    for c in &choices {
        items.push(c.get_self_item().clone());
    }

    ContainerChoiceFrameHandler {
        choices: choices.clone(),
        columns: build_default_columns(),
        item_list_selection: build_list_selection(items.clone(), 1)
    }
}


#[derive(Eq, Hash, PartialEq)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum ContainerChoiceCommand {
    SELECT
}

fn build_default_columns() -> Vec<Column> {
    vec![
        Column {name : "NAME".to_string(), size: 12},
        Column {name : "STORAGE (Kg)".to_string(), size: 12}
    ]
}

fn build_column_text(column: &Column, container: &Container) -> String {
    let item = container.get_self_item();
    match column.name.as_str() {
        "NAME" => {
            item.get_name()
        },
        "STORAGE (Kg)" => {
            format!("{}/{}", container.get_weight_total(), container.get_weight_limit())
        }
        _ => { "".to_string() }
    }
}



impl ContainerChoiceFrameHandler {

}

impl <B : tui::backend::Backend> FrameHandler<B, Vec<Container>> for ContainerChoiceFrameHandler {

    // TODO tidy this up / reduce duplication
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<Vec<Container>>) {
        let frame_size = data.get_frame_size().clone();
        let containers = &self.choices;

        let window_block = Block::default()
            .borders(Borders::ALL);
        let window_area = Rect::new(frame_size.x.clone(), frame_size.y.clone(), frame_size.width.clone(), frame_size.height.clone());
        let inventory_item_lines = window_area.height - 3;
        //self.row_count = inventory_item_lines as i32;
        self.item_list_selection.page_line_count = inventory_item_lines as i32;
        frame.render_widget(window_block, window_area);

        let headings = build_headings(self.columns.clone());
        let headings_area = Rect::new(frame_size.x.clone() + 1, frame_size.y.clone() + 1, frame_size.width.clone() - 4, 2);
        frame.render_widget(headings, headings_area);

        // -3 for the heading and 2  borders
        let mut line_index = 0;
        let _start_index= self.item_list_selection.get_start_index();
        let start_index = 0;
        let end_of_page_representive_index = self.item_list_selection.get_end_of_page_index();

        if !containers.is_empty() {
            let view_contents = &containers[start_index as usize..=end_of_page_representive_index as usize];
            for c in view_contents {
                let item_index = start_index.clone() + line_index.clone();
                let mut x_offset: u16 = frame_size.x.clone() as u16 + 1;
                let y_offset: u16 = frame_size.y.clone() as u16 + 2 + line_index.clone() as u16;
                let current_index = self.item_list_selection.is_focused(item_index);
                let selected = self.item_list_selection.is_selected(item_index);
                for column in &self.columns {
                    let text = build_column_text(column, c);
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
                    frame.render_widget(column_text.clone(), text_area);
                    x_offset += column_length as u16 + 1;
                }
                line_index += 1;
            }

            //let usage_description = build_command_usage_descriptions(&self.commands);
            //let usage_text = build_paragraph(usage_description.clone());
            //let text_area = Rect::new(window_area.x.clone() + 1, window_area.y.clone() + window_area.height.clone() - 1, usage_description.len().try_into().unwrap(), 1);
            //frame.render_widget(usage_text.clone(), text_area);

            // From right hand to left hand side draw the info text
            let page_count = build_page_count(&self.item_list_selection, window_area.clone());
            frame.render_widget(page_count.0, page_count.1);
        }
    }
}

#[derive(Clone)]
pub enum ContainerChoiceFrameHandlerInputResult {
    None,
    Select(Container)
}

impl InputHandler<ContainerChoiceFrameHandlerInputResult> for ContainerChoiceFrameHandler {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<ContainerChoiceFrameHandlerInputResult>, Error> {
        if let Some(key) = input {
            match key {
                Key::Up => {
                    self.item_list_selection.move_up();
                },
                Key::Down => {
                    self.item_list_selection.move_down();
                },
                Key::Char('\n') => {
                    let chosen_index = self.item_list_selection.get_true_index();
                    log::info!("Chosen index: {}", chosen_index);
                    if let Some(c) = self.choices.get(chosen_index as usize) {
                        let container_name = c.get_self_item().get_name();
                        log::info!("Returning input result for Select of: {}", container_name);
                        return Ok(InputResult {
                            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
                            view_specific_result: Some(ContainerChoiceFrameHandlerInputResult::Select(c.clone()))
                        });
                    }
                },
                _ => {}
            }
        }

        return  Ok(InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: Some(ContainerChoiceFrameHandlerInputResult::None)});
    }
}

