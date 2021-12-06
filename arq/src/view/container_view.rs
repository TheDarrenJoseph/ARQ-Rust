use std::io;
use std::io::Error;
use std::convert::TryInto;

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

impl <B : tui::backend::Backend> ContainerView<'_, B> {
    pub(crate) fn begin(&mut self) {
        self.draw();
        while !self.handle_input().unwrap() {
            self.draw();
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
            let padding = build_padding(column.size - name.len() as i8);
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

fn build_cell<'a>(text: String) -> Paragraph<'a> {
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
        let row_count = get_row_count(frame.size().height.clone() as i32, container_len);
        self.row_count = row_count;
        self.item_list_selection.page_line_count = row_count.clone();

        let window_block = Block::default()
            .borders(Borders::ALL)
            .title(container.get_self_item().name.clone());
        let window_area = Rect::new(1, 1, frame.size().width.clone() - 4, frame.size().height.clone() - 4);
        frame.render_widget(window_block, window_area);

        let headings = self.build_headings();
        let headings_area = Rect::new(2, 2, frame.size().width.clone() - 4, 2);
        frame.render_widget(headings, headings_area);

        let mut index = 0;
        for c in container.get_contents() {
            let item = &c.get_self_item();
            let mut offset : u16 = 2;
            for column in &self.columns {
                let text = build_column_text(column, item);
                let current_index = self.item_list_selection.is_current_index(index.clone());
                let selected = self.item_list_selection.is_selected(index.clone());
                let mut cell = build_cell(text);
                if current_index && selected {
                    cell = cell.style(Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED));
                } else  if current_index {
                    cell = cell.style(Style::default().add_modifier(Modifier::REVERSED));
                } else if selected {
                    cell = cell.style(Style::default().fg(Color::Green));
                }

                let column_length = column.size as i8;
                let cell_area = Rect::new( offset.clone(), (3 + index.clone()).try_into().unwrap(), column_length.try_into().unwrap(), 1);
                frame.render_widget(cell.clone(), cell_area);
                offset += column_length as u16;
            }
            index += 1;
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
                    if !self.frame_handler.item_list_selection.get_selected_items().is_empty() {
                        self.frame_handler.item_list_selection.cancel_selection();
                    } else {
                        self.terminal_manager.terminal.clear()?;
                        return Ok(true)
                    }
                },
                Key::Char('o') => {
                    if !self.frame_handler.item_list_selection.is_selecting() {
                        let current_index = self.frame_handler.item_list_selection.get_current_index();
                        let mut item = self.container.get_mut(current_index);
                        if item.can_open() {
                            let mut items = Vec::new();
                            for c in item.get_contents() {
                                let self_item = c.get_self_item();
                                items.push(self_item);
                            }
                            let mut view = build_container_view(item, &mut self.ui, &mut self.terminal_manager);
                            view.begin();
                        }
                    }
                },
                Key::Char('\n') => {
                    self.frame_handler.item_list_selection.toggle_select();
                },
                Key::Char(c) => {},
                Key::Backspace => {},
                Key::Up => {
                    self.frame_handler.item_list_selection.move_up();
                },
                Key::Down => {
                    self.frame_handler.item_list_selection.move_down();
                },
                _ => {}
            }
            return Ok(false)
        }
    }
}