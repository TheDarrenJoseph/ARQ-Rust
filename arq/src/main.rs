use std::io;
use tui::widgets::{Block, Borders};
use tui::style::{Style, Color};
use tui::layout::{Rect};

mod terminal_manager;

fn main<>() -> Result<(), io::Error> {

    let mut terminal_manager = terminal_manager::init::<tui::backend::TermionBackend<termion::raw::RawTerminal<std::io::Stdout>>>().unwrap();
    return terminal_manager.terminal.draw(|frame| {
        let frame_size = frame.size();
        let size = Rect::new(frame_size.x, frame_size.y, frame_size.width-2, frame_size.height-2);
        let block = Block::default()
            .title("ARQ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        frame.render_widget(block, size)
    });
}