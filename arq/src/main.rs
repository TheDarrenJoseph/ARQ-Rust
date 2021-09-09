use std::io;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::style::{Style, Color};
use tui::layout::{Rect};
use std::convert::TryInto;

mod terminal_manager;

fn draw_start_menu<B : tui::backend::Backend>(frame : &mut tui::terminal::Frame<'_, B> ) {
    let frame_size = frame.size();

    let main_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    let main_size = Rect::new(frame_size.x, frame_size.y, frame_size.width-2, frame_size.height-2);
    frame.render_widget(main_block, main_size);

    let menu_titles = ["Play", "Settings", "Info", "Quit"];
    let menu_items : Vec<ListItem> = menu_titles.iter().cloned().map(ListItem::new).collect();
    let menu_list = List::new(menu_items)
        .block(Block::default()
            .borders(Borders::NONE)
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
    let menu_size = Rect::new(4, 4, main_size.width/2, menu_titles.len().try_into().unwrap());
    frame.render_widget(menu_list, menu_size);

}

fn main<>() -> Result<(), io::Error> {
    let mut terminal_manager = terminal_manager::init().unwrap();
    return terminal_manager.terminal.draw(|frame| { draw_start_menu(frame) });
}