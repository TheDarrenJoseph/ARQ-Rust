use std::io;
use std::io::Error;
use tui::layout::{Rect};
use tui::widgets::{Block, Borders};
use termion::input::TermRead;
use termion::event::Key;

use crate::ui::{render_main_window};
use crate::terminal_manager::TerminalManager;
use crate::character::{get_all_attributes, Character};
use crate::widget::{Focusable, Widget, WidgetType, build_dropdown, build_text_input, build_number_input, build_number_input_with_value};

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

    fn build_attribute_inputs(&mut self, mut character: Character) {
        for attribute in get_all_attributes() {
            let attribute_input = build_number_input(true,1, attribute.to_string(), 1);
            self.widgets.push(attribute_input);
        }
        let free_points = build_number_input_with_value(false, character.get_free_attribute_points() as i32, 1, "Free points".to_string(), 1);
        self.widgets.push(free_points);
    }

    fn build_widgets(&mut self, character: Character) {
        let name_input =  build_text_input(12, String::from("Name"), 2);
        self.widgets.push(name_input);

        let class_input = build_dropdown("Class".to_string(), vec!["None".to_string(), "Warrior".to_string()]);
        self.widgets.push(class_input);

        self.build_attribute_inputs(character);

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
                    WidgetType::Number(w) => {
                        let widget_size = Rect::new(6, 6 + offset.clone(), frame_size.width.clone() / 2, 1);
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    },
                    WidgetType::Dropdown(w) => {
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    }
                }

                offset += 1;
            }
        }
    }

    pub fn draw_character_creation<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>, character: Character) {
        log::info!("Drawing character creation...");
        render_main_window(frame);

        let frame_size = frame.size();
        let window_size = Rect::new(4, 4, frame_size.width / 2, frame_size.height / 2);
        let creation_block = Block::default()
        .borders(Borders::ALL)
        .title("Character Creation");
        frame.render_widget(creation_block, window_size);

        if self.widgets.is_empty() {
            log::info!("Building input widgets...");
            self.build_widgets(character);
        }

        let attributes_block = Block::default()
            .borders(Borders::ALL)
            .title("Attributes");
        let attributes_size = Rect::new(5, 7, (frame_size.width.clone() / 2) - 2, (frame_size.height.clone() / 2) - 4);
        frame.render_widget(attributes_block, attributes_size);

        self.draw_text_inputs(frame);
    }
}

impl <B : tui::backend::Backend> CharacterView<'_, B> {
    pub fn draw(&mut self) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        let character = self.character.clone();
        self.terminal_manager.terminal.draw(|frame| { frame_handler.draw_character_creation(frame, character) })?;
        Ok(())
    }

    pub fn update_free_points(&mut self, free_points: i32) {
        for widget in self.frame_handler.widgets.iter_mut() {
            match &mut widget.state_type {
                WidgetType::Number(state) => {
                    if state.name == "Free points" {
                        state.set_input(free_points.clone());
                    }
                },
                _ => {}
            }
        }
    }

    pub fn handle_input(&mut self) -> Result<bool, Error> {
        let key = io::stdin().keys().next().unwrap().unwrap();
        let frame_handler = &mut self.frame_handler;
        let widgets = &mut frame_handler.widgets;

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
                        self.draw()?;
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
                        self.draw()?;
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

                        self.draw()?;
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
                        self.draw()?;
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
                        self.draw()?;
                    },
                    None => {}
                }
            },
            Key::Right => {
                match selected_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Number(state) => {
                                let free_points = self.character.get_free_attribute_points().clone();
                                if free_points > 0 {
                                    state.increment();
                                    self.character.set_free_attribute_points(free_points - 1);
                                    self.update_free_points(free_points.clone() as i32 - 1);
                                    self.draw()?;
                                }
                            },
                            _ => {}
                        }
                    },
                    None => {}
                }
            },
            Key::Left => {
                match selected_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Number(state) => {
                                let free_points = self.character.get_free_attribute_points();
                                if free_points < self.character.get_max_free_attribute_points() {
                                    state.decrement();
                                    self.character.set_free_attribute_points(free_points + 1);
                                    self.update_free_points(free_points.clone() as i32 + 1);
                                }
                                self.draw()?;
                            },
                            _ => {}
                        }
                    },
                    None => {}
                }
            },
            _ => {
            }
        }
        Ok(false)
    }
}
