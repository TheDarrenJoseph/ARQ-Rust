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
use crate::terminal_manager::TerminalManager;
use crate::container::Container;
use crate::items::Item;

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

    ContainerView::<B> { container, ui, terminal_manager, frame_handler: ContainerFrameHandler { selected_index: 0, columns }}
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
    selected_index : i32,
    columns : Vec<Column>
}

impl ContainerFrameHandler {
    fn increment_selection(&mut self) {
        let column_count = self.columns.len() as i32;
        if self.selected_index < column_count - 1 {
            self.selected_index += 1
        }
    }

    fn decrement_selection(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1
        }
    }
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

impl <B : tui::backend::Backend> FrameHandler<B, &mut Container> for ContainerFrameHandler {

    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<&mut Container>) {
        let container = data.unpack();
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
                let mut cell= if index == self.selected_index {
                    build_cell(text).style(Style::default().add_modifier(Modifier::REVERSED))
                } else {
                    build_cell(text)
                };
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
                    self.terminal_manager.terminal.clear()?;
                    return Ok(true)
                },
                Key::Char('o') => {

                    let index = self.frame_handler.selected_index.clone();
                    let mut item = self.container.get_mut(index);
                    if item.can_open() {
                        let mut view = build_container_view(item, &mut self.ui, &mut self.terminal_manager);
                        view.begin();
                    }
                },
                Key::Char('\n') => {},
                Key::Char(c) => {},
                Key::Backspace => {},
                Key::Up => {
                    self.frame_handler.decrement_selection();
                },
                Key::Down => {
                    self.frame_handler.increment_selection();
                },
                _ => {}
            }
            return Ok(false)
        }
    }
}