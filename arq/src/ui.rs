use tui::widgets::{Block, Borders, ListState};
use tui::style::{Style, Color};
use tui::layout::{Rect};
use std::convert::TryInto;
use crate::menu::{Menu,ToList};

pub struct UI {
    pub start_menu : Menu,
    pub settings_menu : Menu,
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

impl std::convert::TryFrom<usize> for SettingsMenuChoice {
    type Error = String;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(SettingsMenuChoice::FogOfWar),
            1 => Ok(SettingsMenuChoice::Quit),
            _ => Err("Failed to convert to StartMenuChoice".to_string())
        }
    }
}


pub trait StartMenu {
    fn draw_start_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>);
    fn draw_settings_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>);
}

fn build_main_block<'a, B : tui::backend::Backend>(_frame : &mut tui::terminal::Frame<'_, B>) -> Block<'a> {
    Block::default()
    .borders(Borders::ALL)
    .title("||ARQ -- ASCII Roguelike Quester||")
    .style(Style::default().bg(Color::Black))
}

fn render_main_window<'a, B : tui::backend::Backend>(frame : &mut tui::terminal::Frame<'_, B>) {
    let main_block = build_main_block(frame);
    let frame_size = frame.size();
    let window_size = Rect::new(frame_size.x, frame_size.y, frame_size.width-2, frame_size.height-2);
    frame.render_widget(main_block, window_size);
}

impl StartMenu for UI {
    fn draw_start_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>){
        render_main_window(frame);

        let mut menu_list_state = ListState::default();
        menu_list_state.select(Some(self.start_menu.selection.try_into().unwrap()));

        let frame_size = frame.size();
        let menu_size = Rect::new(4, 4, frame_size.width/2, self.start_menu.menu_titles.len().try_into().unwrap());
        let menu_list = self.start_menu.to_list();
        frame.render_stateful_widget(menu_list, menu_size, &mut menu_list_state);
    }

    fn draw_settings_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>){
        render_main_window(frame);

        let mut menu_list_state = ListState::default();
        menu_list_state.select(Some(self.settings_menu.selection.try_into().unwrap()));

        let frame_size = frame.size();
        let menu_size = Rect::new(4, 4, frame_size.width/2, self.settings_menu.menu_titles.len().try_into().unwrap());
        let menu_list = self.settings_menu.to_list();
        frame.render_stateful_widget(menu_list, menu_size, &mut menu_list_state);
    }
}

pub fn build_start_menu() -> Menu {
    let titles = vec!["Play".to_owned(), "Settings".to_owned(), "Info".to_owned(), "Quit".to_owned()];
    let prompt = Some("-> ".to_owned());
    let menu = Menu { menu_titles : titles,  highlight_text : prompt, selection : 0, selected: false, exit: false};
    menu
}

pub fn build_settings_menu() -> Menu {
    let titles = vec!["Fog of war".to_owned(), "Close settings".to_owned()];
    let menu = Menu { menu_titles : titles,  highlight_text : None, selection : 0, selected: false, exit: false};
    menu
}