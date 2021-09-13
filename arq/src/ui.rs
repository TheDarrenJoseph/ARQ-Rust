use tui::widgets::{Block, Borders, ListState};
use tui::style::{Style, Color};
use tui::layout::{Rect};
use std::convert::TryInto;
use menu::{ToList};

#[path = "menu.rs"]
pub mod menu;

pub struct UI {
    pub menu : menu::Menu,
    pub frame_size : Option<Rect>
}

pub trait StartMenu {
    fn draw_start_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>);
}

impl StartMenu for UI {
    fn draw_start_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>){
        let frame_size = frame.size();

        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("||ARQ -- ASCII Roguelike Quester||")
            .style(Style::default().bg(Color::Black));
        self.frame_size = Some(Rect::new(frame_size.x, frame_size.y, frame_size.width-2, frame_size.height-2));
        frame.render_widget(main_block,  self.frame_size.unwrap());

        let mut menu_list_state = ListState::default();
        menu_list_state.select(Some(self.menu.selection.try_into().unwrap()));

        let menu_size = Rect::new(4, 4, self.frame_size.unwrap().width/2, self.menu.menu_titles.len().try_into().unwrap());
        let menu_list = self.menu.to_list();
        frame.render_stateful_widget(menu_list, menu_size, &mut menu_list_state);
    }
}

pub fn build_start_menu() -> menu::Menu {
    let titles = vec!["Play".to_owned(), "Settings".to_owned(), "Info".to_owned(), "Quit".to_owned()];
    let prompt = Some("-> ".to_owned());
    let menu = menu::Menu { menu_titles : titles,  highlight_text : prompt, selection : 0, selected: false, exit: false};
    menu
}
