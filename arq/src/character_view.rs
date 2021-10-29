use std::io;
use tui::buffer::Cell;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Style};
use tui::text::{Spans,Span};
use tui::widgets::{Block, Borders, ListState, Paragraph, Wrap};
use termion::input::TermRead;

use std::io::Error;
use tui::widgets::StatefulWidget;

use crate::map::Map;
use crate::ui::{render_main_window};
use crate::terminal_manager::TerminalManager;
use crate::colour_mapper;
use crate::character::Character;
use crate::widget::{TextInput, TextInputState};

pub struct CharacterView<'a, B : tui::backend::Backend> {
    pub character : Character,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub text_widgets : Vec<TextInput>
}

pub fn draw_text_inputs<B : tui::backend::Backend>(frame: &mut tui::terminal::Frame<B>) {
    let frame_size = frame.size();

    let name_input_size = Rect::new(5, 5, frame_size.width.clone() / 2, 1);
    let name_input = TextInput { name: String::from("Name"), input_padding: 2, length: 12, selected: false, selected_index: 0};
    let mut name_input_state = TextInputState { input: "".to_string() };
    frame.render_stateful_widget(name_input, name_input_size, &mut name_input_state);

    let class_input_size = Rect::new(5, 6, frame_size.width.clone() / 2, 1);
    let class_input = TextInput { name: String::from("Class"), input_padding: 1, length: 12, selected: false, selected_index: 0};
    let mut class_input_state = TextInputState { input: "".to_string() };
    frame.render_stateful_widget(class_input, class_input_size, &mut class_input_state);
}

pub fn draw_character_creation<B : tui::backend::Backend>(frame: &mut tui::terminal::Frame<B>) {
    log::info!("Drawing character creation...");
    render_main_window(frame);
    let frame_size = frame.size();
    let menu_size = Rect::new(4, 4, frame_size.width / 2, frame_size.height / 2);

    let creation_block = Block::default()
        .borders(Borders::ALL)
        .title("Character Creation")
        .style(Style::default().bg(Color::Black));

    frame.render_widget(creation_block, menu_size);
    draw_text_inputs(frame);
}

impl <B : tui::backend::Backend> CharacterView<'_, B> {

    pub fn draw(&mut self) -> Result<(), Error> {
        self.terminal_manager.terminal.draw(|frame| { draw_character_creation(frame) });
        let key = io::stdin().keys().next().unwrap().unwrap();
        Ok(())
    }


}
