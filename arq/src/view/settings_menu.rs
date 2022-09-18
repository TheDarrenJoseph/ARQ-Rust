use std::io::Error;
use termion::event::Key;
use tui::layout::Rect;
use crate::map::position::Area;

use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::UI;
use crate::view::{GenericInputResult, resolve_input, View};
use crate::widget::{Focusable, Widget, WidgetType};
use crate::widget::widgets::WidgetList;

pub struct SettingsMenu<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub selected_widget: Option<i8>,
    pub widgets: WidgetList
}

impl <'b, B : tui::backend::Backend> View<'b, GenericInputResult> for SettingsMenu<'_, B>  {
    fn begin(&mut self)  -> Result<bool, Error> {
        // Select the first widget
        if self.widgets.widgets.len() > 0 {
            self.widgets.widgets[0].state_type.focus();
        }

        self.terminal_manager.terminal.clear()?;
        self.draw(None)?;

        while !self.handle_input(None).unwrap() {
            self.draw(None)?;
        }
        Ok(true)
    }

    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let terminal = &mut self.terminal_manager.terminal;
        let widgets = &self.widgets;
        terminal.draw(|frame| {
            let frame_size = frame.size();
            let mut offset = 0;
            for widget in widgets.widgets.iter() {
                let widget_area = Rect::new(5, 5 + offset.clone(), frame_size.width.clone() / 2, 1);
                match &widget.state_type {
                    WidgetType::Text(w) => {
                        frame.render_stateful_widget(w.clone(), widget_area, &mut w.clone());
                    },
                    WidgetType::Boolean(w) => {
                        frame.render_stateful_widget(w.clone(), widget_area, &mut w.clone());
                    },
                    _ => {}
                    }
                offset += 1;
            }
        })?;
        Ok(())
    }

    fn handle_input(&mut self, input: Option<Key>) -> Result<bool, Error> {
        let key = resolve_input(input);
        let mut target_widget = None;
        match self.widgets.selected_widget {
            Some(idx) => {
                target_widget = Some(&mut self.widgets.widgets[idx as usize]);
            },
            None => {}
        }

        match key {
            Key::Down => {
                self.widgets.next_widget();
            },
            Key::Up => {
                self.widgets.previous_widget();
            },
            Key::Char('\n') => {
                match target_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Boolean(state) => {
                                state.value = !state.value;
                            }
                            _ => {}
                        }
                    },
                    None => {}
                }
            },
            Key::Backspace => {
                match target_widget {
                    Some(widget) => {
                        match &mut widget.state_type {
                            WidgetType::Text(state) => {
                                state.delete_char();
                            }
                            _ => {}
                        }
                    },
                    None => {}
                }
            }
            Key::Char('q') => {
                return Ok(true)
            },
            Key::Char(c) => {
                match target_widget {
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
            }
            _ => {
            }
        }
        Ok(false)
    }
}