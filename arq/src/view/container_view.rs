use std::io;
use std::io::Error;
use std::convert::TryInto;
use std::collections::VecDeque;

use tui::layout::{Alignment, Rect};
use tui::style::{Style, Color, Modifier};
use tui::buffer::{Buffer};
use tui::widgets::{Block, Borders, ListState, Paragraph, Wrap};
use tui::text::{Spans,Span};
use termion::input::TermRead;
use termion::event::Key;

use crate::ui::{UI, FrameHandler, FrameData};
use crate::view::View;
use crate::terminal::terminal_manager::TerminalManager;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::list_selection::{ListSelection, ItemListSelection, build_list_selection};

pub struct ContainerView<'a, B : tui::backend::Backend> {
    pub container : &'a mut Container,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler : ContainerFrameHandler
}

pub fn build_container_view<'a, B : tui::backend::Backend> (container: &'a mut Container, ui: &'a mut  UI, terminal_manager: &'a mut TerminalManager<B>) -> ContainerView<'a, B> {
    let columns = vec![
        Column {name : "NAME".to_string(), size: 12},
        Column {name : "WEIGHT (Kg)".to_string(), size: 12},
        Column {name : "VALUE".to_string(), size: 12}
    ];

    let mut items = Vec::new();
    // Clone the self items for everything in the container
    for c in container.get_contents() {
        items.push(c.get_self_item().clone());
    }
    ContainerView::<B> { container, ui, terminal_manager,
        frame_handler: ContainerFrameHandler {
            columns,
            row_count: 1,
            item_list_selection: build_list_selection(items, 1)
        }
    }
}

trait ContainerViewCommands {

}

impl <B : tui::backend::Backend> ContainerView<'_, B> {
    pub(crate) fn begin(&mut self) {
        self.draw();
        while !self.handle_input().unwrap() {
            self.draw();
        }
    }

    pub fn rebuild_selection(&mut self, container: &Container) {
        let mut items = Vec::new();
        // Clone the self items for everything in the container
        for c in container.get_contents() {
            items.push(c.get_self_item().clone());
        }
        self.frame_handler.item_list_selection = build_list_selection(items, 1);
    }

    fn get_selected_items(&self) -> Vec<Item> {
        Vec::from(self.frame_handler.item_list_selection.get_selected_items().clone())
    }

    fn find_container_for_item(&self, item: &Item) -> Option<&Container> {
        self.container.find(item)
    }

    fn find_selected_containers(&self, selected_items : Vec<Item>) -> Vec<Container> {
        let mut found = Vec::new();
        for selected_item in selected_items.iter() {
            // Collect our 'Container' wrappers matching the selection
            if let Some(found_container) = self.find_container_for_item(&selected_item) {
                found.push(found_container.clone());
            }
        }
        found
    }

    fn get_selected_containers(&self) -> Vec<Container> {
        let selected_items = self.get_selected_items();
        let selected_containers = self.find_selected_containers(selected_items);
        selected_containers
    }

    fn handle_quit(&mut self) -> Result<bool, Error> {
        if !self.frame_handler.item_list_selection.get_selected_items().is_empty() {
            self.frame_handler.item_list_selection.cancel_selection();
            Ok(false)
        } else {
            self.terminal_manager.terminal.clear()?;
            Ok(true)
        }
    }

    fn toggle_select(&mut self) {
        self.frame_handler.item_list_selection.toggle_select();
    }

    fn move_up(&mut self) {
        self.frame_handler.item_list_selection.move_up();
    }

    fn move_down(&mut self) {
        self.frame_handler.item_list_selection.move_down();
    }

    fn page_up(&mut self) {
        self.frame_handler.item_list_selection.page_up();
    }

    fn page_down(&mut self) {
        self.frame_handler.item_list_selection.page_down();
    }

    fn open_focused(&mut self) {
        if !self.frame_handler.item_list_selection.is_selecting() {
            if let Some(focused_item) = self.frame_handler.item_list_selection.get_focused_item() {
                if focused_item.is_container() {
                    if let Some(focused_container) = self.container.find_mut(focused_item) {
                        let mut items = Vec::new();
                        for c in focused_container.get_contents() {
                            let self_item = c.get_self_item();
                            items.push(self_item);
                        }
                        let mut view = build_container_view(focused_container, &mut self.ui, &mut self.terminal_manager);
                        view.begin();
                    }
                }
            }
        }
    }

    fn move_selection(&mut self) {
        let list_selection = &self.frame_handler.item_list_selection;
        let selected_container_items = &self.get_selected_containers();
        let mut updated = false;
        if list_selection.is_selecting() {
            let focused_item_result = list_selection.get_focused_item();
            let true_index = list_selection.get_true_index();
            if let Some(focused_item) = focused_item_result {
                // Make sure we've not focused any of the selected items
                let focused_items = selected_container_items.iter().find(|ci| ci.get_self_item().get_id() == focused_item.get_id());
                if let None = focused_items {
                    if focused_item.is_container() {
                        if let Some(focused_container) = self.container.find_mut(focused_item) {
                            // Move items into the container
                            focused_container.push(selected_container_items.clone());
                            self.container.remove(selected_container_items.to_vec());
                        }
                    } else {
                        // Move items to this location
                        self.container.remove(selected_container_items.to_vec());
                        self.container.insert(true_index as usize - selected_container_items.len(), selected_container_items.clone());
                    }
                    &mut self.frame_handler.item_list_selection.cancel_selection();
                    updated = true;
                }
            }
        }

        if updated {
            self.rebuild_selection(&self.container.clone());
        }
    }
}

pub struct Column {
    pub name : String,
    pub size : i8
}

pub struct ContainerFrameHandler {
    columns : Vec<Column>,
    row_count: i32,
    pub item_list_selection : ItemListSelection
}

fn build_padding(length : i8) -> String {
    let mut s = String::new();
    for i in 1..length {
        s.push(' ');
    }
    s
}

impl ContainerFrameHandler {
    fn build_headings(&self) -> Paragraph {
        let mut heading_spans = Vec::new();
        let mut spans = Vec::new();
        for column in &self.columns {
            let name = column.name.clone();
            let padding = build_padding(column.size - name.len() as i8 + 2);
            spans.push(Span::raw(column.name.clone()));
            spans.push(Span::raw(padding));
        }
        heading_spans.push(Spans::from(spans));
        Paragraph::new(heading_spans)
            .block(Block::default()
                .borders(Borders::NONE))
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
    }
}

fn build_column_text(column: &Column, item: &Item) -> String {
    match column.name.as_str() {
        "NAME" => {
            item.get_name()
        },
        "WEIGHT (Kg)" => {
            item.get_weight().to_string()
        },
        "VALUE" => {
            item.get_value().to_string()
        },
        _ => { "".to_string() }
    }
}

fn build_paragraph<'a>(text: String) -> Paragraph<'a> {
    let spans = vec![Spans::from(Span::raw(text.clone()))];
    let spans_len = spans.len() as u16;
    let paragraph = Paragraph::new(spans)
        .style(Style::default())
        .alignment(Alignment::Left);
    paragraph
}

fn get_row_count(frame_height: i32, container_len: i32) -> i32 {
    let available_frame_rows = frame_height - 2;
    if container_len < available_frame_rows {
        container_len
    } else {
        available_frame_rows
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, &mut Container> for ContainerFrameHandler {

    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<&mut Container>) {
        let container = data.unpack();
        let container_len = container.get_contents().len() as i32;

        let window_block = Block::default()
            .borders(Borders::ALL)
            .title(container.get_self_item().name.clone());
        let window_area = Rect::new(1, 1, frame.size().width.clone() - 4, frame.size().height.clone() - 4);
        let inventory_item_lines = window_area.height - 3;
        self.row_count = inventory_item_lines as i32;
        self.item_list_selection.page_line_count = inventory_item_lines as i32;
        frame.render_widget(window_block, window_area);

        let headings = self.build_headings();
        let headings_area = Rect::new(2, 2, frame.size().width.clone() - 4, 2);
        frame.render_widget(headings, headings_area);

        // -3 for the heading and 2  borders
        let mut line_index = 0;
        let start_index= self.item_list_selection.get_start_index();
        let end_of_page_representive_index = self.item_list_selection.get_end_of_page_index();

        if !container.get_contents().is_empty() {
            let view_contents = &container.get_contents()[start_index as usize..=end_of_page_representive_index as usize];
            for c in view_contents {
                let item_index = start_index.clone() + line_index.clone();
                let item = &c.get_self_item();
                let mut offset: u16 = 2;
                let current_index = self.item_list_selection.is_focused(item_index);
                let selected = self.item_list_selection.is_selected(item_index);
                for column in &self.columns {
                    let text = build_column_text(column, item);
                    let mut column_text = build_paragraph(text);
                    if current_index.clone() && selected.clone() {
                        column_text = column_text.style(Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED));
                    } else if current_index {
                        column_text = column_text.style(Style::default().add_modifier(Modifier::REVERSED));
                    } else if selected {
                        column_text = column_text.style(Style::default().fg(Color::Green));
                    }

                    let column_length = column.size as i8;
                    let text_area = Rect::new(offset.clone(), (3 + line_index.clone()).try_into().unwrap(), column_length.try_into().unwrap(), 1);
                    frame.render_widget(column_text.clone(), text_area);
                    offset += column_length as u16 + 1;
                }
                line_index += 1;
            }

            let usage_description = "(o)pen, (d)rop, (m)ove";
            let mut usage_text = build_paragraph(String::from(usage_description));
            let text_area = Rect::new( window_area.x.clone() + 1, window_area.height.clone(), usage_description.len().try_into().unwrap(), 1);
            frame.render_widget(usage_text.clone(), text_area);


            let page_number = self.item_list_selection.get_page_number();
            let total_pages = self.item_list_selection.get_total_pages();

            let page_count_text = format!("Page {}/{}", page_number, total_pages);
            let width = page_count_text.len().try_into().unwrap();
            let page_count_paragraph = build_paragraph(page_count_text);
            let page_count_area = Rect::new( window_area.x.clone() + 1 + usage_description.len() as u16 + 2 , window_area.height.clone(), width, 1);
            frame.render_widget(page_count_paragraph, page_count_area);
        }
    }
}

impl <B : tui::backend::Backend> View for ContainerView<'_, B> {
    fn draw(&mut self) -> Result<(), Error> {
        let ui = &mut self.ui;
        let terminal =  &mut self.terminal_manager.terminal;
        let container = &mut (*self.container);
        let frame_handler = &mut self.frame_handler;
        terminal.draw(|frame| {
            ui.render(frame);
            frame_handler.handle_frame(frame, FrameData { data: container });
        })?;

        Ok(())
    }

    fn handle_input(&mut self) -> Result<bool, Error> {
        loop {
            let key = io::stdin().keys().next().unwrap().unwrap();
            match key {
                Key::Char('q') => {
                    if self.handle_quit()? {
                        return Ok(true)
                    }
                },
                Key::Char('o') => {
                    self.open_focused();
                },
                Key::Char('m') => {
                    self.move_selection();
                },
                Key::Char('\n') => {
                    self.toggle_select();
                },
                Key::Char(c) => {},
                Key::Backspace => {},
                Key::Up => {
                    self.move_up();
                },
                Key::PageUp => {
                    self.page_up();
                },
                Key::Down => {
                    self.move_down();
                },
                Key::PageDown => {
                    self.page_down();
                },
                _ => {}
            }
            return Ok(false)
        }
    }
}