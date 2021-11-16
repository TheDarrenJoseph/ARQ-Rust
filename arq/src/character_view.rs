use std::io;
use std::io::Error;
use tui::layout::{Rect};
use tui::widgets::{Block, Borders};
use termion::input::TermRead;
use termion::event::Key;

use crate::ui::{render_main_window};
use crate::terminal_manager::TerminalManager;
use crate::character::{get_all_attributes, Character, Race, Class, determine_class, Attribute};
use crate::widget::text_widget::build_text_input;
use crate::widget::dropdown_widget::{build_dropdown, DropdownInputState};
use crate::widget::number_widget::{build_number_input, build_number_input_with_value, NumberInputState};
use crate::widget::button_widget::build_button;
use crate::widget::{Focusable, Widget, WidgetType, Named};

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
        let mut scores = character.get_attribute_scores();
        for attribute in get_all_attributes() {
            let score = scores.iter_mut().find(|score| score.attribute == attribute);
            let mut attribute_input = build_number_input(true,1, attribute.to_string(), 1);
            match attribute_input.state_type {
                WidgetType::Number(ref mut state) => {
                    match score {
                        Some(s) => {
                            state.set_input(s.score.into());
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
            self.widgets.push(attribute_input);
        }
        let free_points = build_number_input_with_value(false, character.get_free_attribute_points() as i32, 1, "Free points".to_string(), 1);
        self.widgets.push(free_points);
    }

    fn build_widgets(&mut self, mut character: Character) {
        let name_input =  build_text_input(12, String::from("Name"), character.get_name(), 2);
        self.widgets.push(name_input);

        let mut class_input = build_dropdown("Class".to_string(), vec!["None".to_string(), "Warrior".to_string()]);
        match class_input.state_type {
            WidgetType::Dropdown(ref mut state) => {
                state.select(character.get_class().to_string())
            }, _ => {}
        }
        self.widgets.push(class_input);

        self.build_attribute_inputs(character);

        let button = build_button("[Enter]".to_string().len() as i8, "[Enter]".to_string());
        self.widgets.push(button);

        self.selected_widget = Some(0);
        &mut self.widgets[0].state_type.focus();
    }

    pub fn draw_inputs<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>) {
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
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    },
                    WidgetType::Dropdown(w) => {
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    },
                    WidgetType::Button(w) => {
                        frame.render_stateful_widget(w.clone(), widget_size, &mut w.clone());
                    }
                }
                offset += 1;
            }
        }
    }

    fn draw_character_details<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>, character: Character, title: String) {
        render_main_window(frame);

        let frame_size = frame.size();
        let window_size = Rect::new(4, 4, frame_size.width / 2, frame_size.height / 2);
        let creation_block = Block::default()
            .borders(Borders::ALL)
            .title(title);
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

        self.draw_inputs(frame);
    }

    pub fn draw_character_creation<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>, character: Character) {
        log::info!("Drawing character creation...");
        self.draw_character_details(frame, character, "Character Creation".to_string());
    }

    pub fn draw_character_info<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>, character: Character) {
        log::info!("Drawing character details...");
        let name = character.get_name().clone();
        self.draw_character_details(frame, character,name);
    }
}

impl <B : tui::backend::Backend> CharacterView<'_, B> {
    pub fn draw_creation(&mut self) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        let character = self.character.clone();
        self.terminal_manager.terminal.draw(|frame| { frame_handler.draw_character_creation(frame, character) })?;
        Ok(())
    }

    pub fn draw_details(&mut self) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        let character = self.character.clone();
        self.terminal_manager.terminal.draw(|frame| { frame_handler.draw_character_info(frame, character) })?;
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
                            WidgetType::Button(state) => {
                                match state.get_name().as_str() {
                                    "[Enter]" => {
                                        return Ok(true)
                                    },
                                    _ => {}
                                }
                            },
                            _ => {
                            }
                        }
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
                            }
                            _ => {}
                        }

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

    pub fn get_character(&mut self) -> Character {
        let mut character = self.character.clone();
        let mut scores  = character.get_attribute_scores();
        for widget in self.frame_handler.widgets.iter_mut() {
            let mut state_type = &mut widget.state_type;
            if String::from("Name") == state_type.get_name() {
                match state_type {
                    WidgetType::Text(state) => {
                        character.set_name(state.get_input());
                    },
                    _ => {}
                }
            }

            if String::from("Class") == state_type.get_name() {
                match state_type {
                    WidgetType::Dropdown(state) => {
                        let class = determine_class(state.get_selection());
                        match class {
                            Some(c) => {
                                character.set_class(c);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        }

        let mut number_states : Vec<NumberInputState> = Vec::new();
        let ns_options : Vec<Option<NumberInputState>> = self.frame_handler.widgets.iter_mut().map(|w| map_state(  w)).collect();
        for ns_option in ns_options {
            match ns_option {
                Some(ns) => {
                    number_states.push(ns)
                },
                _ => {}
            }
        }

        for attribute in get_all_attributes() {
            let number_state = number_states.iter_mut().find(|ns| ns.name == attribute.to_string());
            match number_state {
                Some(mut ns) => {
                    let mut score = scores.iter_mut().find(|score| score.attribute == attribute);
                    match score {
                        Some(s) => {
                            s.score = ns.get_input() as i8;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        character.set_attribute_scores(scores);
        character
    }
}

fn map_state(widget : &mut Widget) -> Option<NumberInputState> {
    match &widget.state_type {
        WidgetType::Number(state) => {
            Some(state.clone())
        },
        _ => {
            None
        }
    }
}