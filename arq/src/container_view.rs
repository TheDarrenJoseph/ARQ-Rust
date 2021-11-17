use std::io;
use std::io::Error;

use tui::layout::{Rect};
use tui::style::{Style, Color};
use tui::buffer::{Buffer};
use tui::widgets::{Block, Borders};
use termion::input::TermRead;
use termion::event::Key;

use crate::ui::{UI};
use crate::view::View;
use crate::terminal_manager::TerminalManager;
use crate::container::Container;

pub struct ContainerView<'a, B : tui::backend::Backend> {
    pub container : &'a mut Container,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>
}

impl <B : tui::backend::Backend> View for ContainerView<'_, B> {
    fn draw(&mut self) -> Result<(), Error> {
        let ui = &mut self.ui;
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