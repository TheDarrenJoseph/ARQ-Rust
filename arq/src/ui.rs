use tui::widgets::{Block, Borders, ListState};
use tui::style::{Style, Color};
use tui::layout::{Rect};
use std::convert::TryInto;
use crate::ui::menu::ToList;

#[path = "menu.rs"]
pub mod menu;

pub struct UI {
    pub start_menu : menu::Menu<StartMenuChoice>,
    pub frame_size : Option<Rect>
}

pub enum StartMenuChoice {
    Play,
    Settings,
    Info,
    Quit
}

impl std::convert::TryFrom<usize> for StartMenuChoice {
    type Error = String;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(StartMenuChoice::Play),
            1 => Ok(StartMenuChoice::Settings),
            2 => Ok(StartMenuChoice::Info),
            3 =>  Ok(StartMenuChoice::Quit),
            _ => Err("Failed to convert to StartMenuChoice".to_string())
        }
    }
}

pub enum SettingsMenuChoice {
    FogOfWar,
    Quit
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
        menu_list_state.select(Some(self.start_menu.selection.try_into().unwrap()));

        let menu_size = Rect::new(4, 4, self.frame_size.unwrap().width/2, self.start_menu.menu_titles.len().try_into().unwrap());
        let menu_list = self.start_menu.to_list();
        frame.render_stateful_widget(menu_list, menu_size, &mut menu_list_state);
    }
}

pub fn build_start_menu() -> menu::Menu<StartMenuChoice> {
    let titles = vec!["Play".to_owned(), "Settings".to_owned(), "Info".to_owned(), "Quit".to_owned()];
    let prompt = Some("-> ".to_owned());
    let menu = menu::Menu { menu_titles : titles,  highlight_text : prompt, selection : 0, choice: None, selected: false, exit: false};
    menu
}

pub fn build_settings_menu() -> menu::Menu<SettingsMenuChoice> {
    let titles = vec!["Fog of war".to_owned(), "Close settings".to_owned()];
    let menu = menu::Menu { menu_titles : titles,  highlight_text : None, selection : 0, choice: None, selected: false, exit: false};
    menu
}