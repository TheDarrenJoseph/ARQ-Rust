use std::io;
use std::io::Error;
use tui::buffer::Cell;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Style};
use tui::text::{Spans,Span};
use tui::widgets::{Block, Borders, ListState, Paragraph, Wrap};
use termion::input::TermRead;
use termion::event::Key;
use tui::widgets::StatefulWidget;

use crate::map::Map;
use crate::ui::{render_main_window};
use crate::terminal_manager::TerminalManager;
use crate::colour_mapper;
use crate::character::Character;
use crate::widget::{Focusable, Widget, WidgetType, TextInputState, DropdownInputState, build_dropdown, build_text_input};

pub struct CharacterView<'a, B : tui::backend::Backend> {
    pub character : Character,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: CharacterViewFrameHandler
}

pub struct CharacterViewFrameHandler {
    pub selected_widget: Option<i8>,
    pub widgets: Vec<Widget>
}

impl CharacterViewFrameHandler {

    fn previous_widget(&mut self) {
        let selected_widget = self.selected_widget.unwrap();
        if selected_widget > 0 && selected_widget < self.widgets.len() as i8 {
            self.select_widget(selected_widget - 1);
        }
    }

    fn next_widget(&mut self) {
        let selected_widget = self.selected_widget.unwrap();
        if selected_widget >= 0 && selected_widget < self.widgets.len() as i8 - 1 {
            self.select_widget(selected_widget + 1);
        }
    }

    fn select_widget(&mut self, index: i8) {
        let mut offset = 0;
        for widget in self.widgets.iter_mut() {
            if offset == index {
                self.selected_widget =  Some(offset.clone());
                widget.state_type.focus();
            } else {
                widget.state_type.unfocus();
            }
            offset += 1;
        }
    }

    fn build_text_inputs(&mut self) {
        let name_input =  build_text_input(12, String::from("Name"), 2);
        self.widgets.push(name_input);

        let class_input = build_dropdown("Class".to_string(), vec!["None".to_string(), "Warrior".to_string()]);
        self.widgets.push(class_input);
        self.selected_widget = Some(0);
        &mut self.widgets[0].state_type.focus();
    }

    pub fn draw_text_inputs<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>) {
        let frame_size = frame.size();
        if self.widgets.len() > 0 {
            let mut offset = 0;
            for widget in self.widgets.iter_mut() {
                let widget_size = Rect::new(5, 5 + offset.clone(), frame_size.width.clone() / 2, 1);

                match &mut widget.state_type {
                    WidgetType::Text(w) => {
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    },
                    WidgetType::Dropdown(w) => {
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    },
                    _ => {}
                }

                offset += 1;
            }
        }
    }

    pub fn draw_character_creation<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>) {
        log::info!("Drawing character creation...");
        render_main_window(frame);
        let frame_size = frame.size();
        let menu_size = Rect::new(4, 4, frame_size.width / 2, frame_size.height / 2);

        let creation_block = Block::default()
        .borders(Borders::ALL)
        .title("Character Creation")
        .style(Style::default().bg(Color::Black));

        frame.render_widget(creation_block, menu_size);
        if self.widgets.is_empty() {
            log::info!("Building input widgets...");
            self.build_text_inputs();
        }
        self.draw_text_inputs(frame);
    }
}

impl <B : tui::backend::Backend> CharacterView<'_, B> {
    pub fn draw(&mut self) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        self.terminal_manager.terminal.draw(|frame| { frame_handler.draw_character_creation(frame) });
        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<bool, Error> {
        let key = io::stdin().keys().next().unwrap().unwrap();
        let frame_handler = &mut self.frame_handler;
        let mut widgets = &mut frame_handler.widgets;

        let mut selected_widget = None;
        match frame_handler.selected_widget {
            Some(idx) => {
                let widget = &mut widgets[idx as usize];
                widget.state_type.focus();
                selected_widget = Some(widget);
            },
            None => {}
        }

        match key {
            Key::Char('q') => {
                self.terminal_manager.terminal.clear()?;
                return Ok(true)
            },
            Key::Char('\n') => {
                match selected_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Dropdown(state) => {
                                state.toggle_show();
                            },
                            _ => {
                            }
                        }
                        self.draw();
                    }
                    None => {}
                }
            },
            Key::Char(c) => {
                match selected_widget {
                    Some(widget) => {
                        log::info!("Input: {}", c.to_string());

                        match &mut widget.state_type {
                            WidgetType::Text(state) => {
                                state.add_char(c);
                                log::info!("Widget state input is: {}", state.get_input());
                            },
                            _ => {}
                        }
                        self.draw();
                    },
                    None => {}
                }
            },
            Key::Backspace => {
                match selected_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Text(state) => {
                                state.delete_char();
                            },
                            _ => {}
                        }

                        self.draw();
                    }
                    None => {}
                }
            },
            Key::Down => {
                match selected_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Dropdown(state) => {
                                if state.is_showing_options() {
                                    state.select_next();
                                } else {
                                    frame_handler.next_widget();
                                }
                            },
                            _ => {
                                frame_handler.next_widget();
                            }
                        }
                        self.draw();
                    }
                    None => {}
                }
            },
            Key::Up => {
                match selected_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Dropdown(state) => {
                                if state.is_showing_options() {
                                    state.select_previous();
                                } else {
                                    frame_handler.previous_widget();
                                }
                            },
                            _ => {
                                frame_handler.previous_widget();
                            }
                        }
                        self.draw();
                    }
                    None => {}
                }
            },
            _ => {
            }
        }
        Ok(false)
    }
}
