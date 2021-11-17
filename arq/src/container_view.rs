use std::io;
use std::io::Error;
use std::convert::TryInto;

use tui::layout::{Alignment, Rect};
use tui::style::{Style, Color};
use tui::buffer::{Buffer};
use tui::widgets::{Block, Borders, ListState, Paragraph, Wrap};
use tui::text::{Spans,Span};
use termion::input::TermRead;
use termion::event::Key;

use crate::ui::{UI, FrameHandler, FrameData};
use crate::view::View;
use crate::terminal_manager::TerminalManager;
use crate::container::Container;

pub struct ContainerView<'a, B : tui::backend::Backend> {
    pub container : &'a mut Container,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler : ContainerFrameHandler,
}

pub fn build_container_view<'a, B : tui::backend::Backend> (container: &'a mut Container, ui: &'a mut  UI, terminal_manager: &'a mut TerminalManager<B>) -> ContainerView<'a, B> {
    let columns = vec![
        Column {name : "NAME".to_string(), size: 32},
        Column {name : "WEIGHT (Kg)".to_string(), size: 13},
        Column {name : "VALUE".to_string(), size: 13}
    ];
    ContainerView::<B> { container, ui, terminal_manager, frame_handler: ContainerFrameHandler { columns }}
}

pub struct Column {
    pub name : String,
    pub size : i8
}

pub struct ContainerFrameHandler {
    columns : Vec<Column>
}

fn build_padding(length : i8) -> String {
    let mut s = String::new();
    for i in 1..length {
        s.push(' ');
    }
    s
}

impl <B : tui::backend::Backend> FrameHandler<B, &mut Container> for ContainerFrameHandler{
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<&mut Container>) {
        let container = data.unpack();
        let window_block = Block::default()
            .borders(Borders::ALL)
            .title(container.get_self_item().name.clone());
        let window_area = Rect::new(1, 1, frame.size().width.clone() - 4, frame.size().height.clone() - 4);
        frame.render_widget(window_block, window_area);

        let headings_area = Rect::new(2, 2, frame.size().width.clone() - 4, 2);
        let mut heading_spans = Vec::new();
        let mut spans = Vec::new();
        for column in &self.columns {
            let name = column.name.clone();
            let padding = build_padding(column.size - name.len() as i8);
            spans.push(Span::raw(column.name.clone()));
            spans.push(Span::raw(padding));
        }
        heading_spans.push(Spans::from(spans));

        let headings = Paragraph::new(heading_spans)
            .block(Block::default()
                .borders(Borders::NONE))
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        frame.render_widget(headings, headings_area);

        let mut index = 0;
        let item_count = container.get_contents().len() as u16;
        for item in container.get_contents() {
            let spans = vec![Spans::from(Span::raw(index.to_string()))];
            let spans_len = spans.len() as u16;
            let row = Paragraph::new(spans)
                .block(Block::default()
                    .borders(Borders::NONE))
                .style(Style::default())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });

            let frame_size = frame.size();
            let row_area = Rect::new(2, 3 + index.clone(), 12, 2);
            frame.render_widget(row, row_area);
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
        let key = io::stdin().keys().next().unwrap().unwrap();
        match key {
            Key::Char('q') => {
                self.terminal_manager.terminal.clear()?;
                return Ok(true)
            },
            Key::Char('\n') => {
            },
            Key::Char(c) => {
            },
            Key::Backspace => {
            },
            _ => {
            }
        }
        Ok(false)
    }
}