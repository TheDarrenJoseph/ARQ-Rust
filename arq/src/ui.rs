use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::style::{Style, Color};
use tui::layout::{Rect};
use std::convert::TryInto;

pub fn draw_start_menu<B : tui::backend::Backend>(frame : &mut tui::terminal::Frame<'_, B> ) {
    let frame_size = frame.size();

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title("||ARQ -- ASCII Roguelike Quester||")
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
        .highlight_symbol("-> ")
        .highlight_style(Style::default().fg(Color::Red));

    let mut menu_list_state = ListState::default();
    menu_list_state.select(Some(0));

    let menu_size = Rect::new(4, 4, main_size.width/2, menu_titles.len().try_into().unwrap());
    frame.render_stateful_widget(menu_list, menu_size, &mut menu_list_state);

}