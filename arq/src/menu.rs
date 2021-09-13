use std::io;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::style::{Style, Color};
use termion::{event::Key};
use termion::input::TermRead;

pub struct Menu{
    pub menu_titles: Vec<String>,
    pub highlight_text: Option<String>,
    pub selection: usize,
    pub selected : bool,
    pub exit : bool
}

pub trait ToList {
   fn to_list(&self) -> List;
}

pub trait Selection {
    fn select_up(&mut self);
    fn select_down(&mut self);
    fn handle_input(&mut self);
}

impl ToList for Menu {
    fn to_list(&self) -> List {
        let menu_items: Vec<ListItem> = self.menu_titles.iter().cloned().map(ListItem::new).collect();
        let mut list = List::new(menu_items)
            .block(Block::default()
                .borders(Borders::NONE)
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Red));

        if self.highlight_text != None {
            list = list.highlight_symbol(self.highlight_text.as_ref().unwrap());
        }
        list
    }
}

impl Selection for Menu {
    fn select_up(&mut self) {
        if self.selection > 0 && self.selection < self.menu_titles.len() {
            self.selection -= 1;
        }
    }

    fn select_down(&mut self) {
        if self.selection < self.menu_titles.len() {
            self.selection += 1;
        }
    }

    fn handle_input(&mut self) {
        self.selected = false;
        let input = io::stdin();
        let keys = input.keys();
        for key in keys {
            match key.unwrap() {
                Key::Char('q') => {
                    self.exit = true;
                    break;
                }
                Key::Char('\n') => {
                    self.selected = true;
                    break;
                }
                Key::Char(c) => {
                    log::info!("Inputted: '{}'", c);
                }
                Key::Up => {
                    self.select_up();
                    break;
                }
                Key::Down => {
                    self.select_down();
                    break;
                }
                Key::Right => {
                    self.selected = true;
                    break;
                }
                _ => {
                    log::info!("Unknown");
                }
            }
        }

    }
}
