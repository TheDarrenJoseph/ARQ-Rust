use std::io::Error;
use termion::event::Key;
use tui::layout::Rect;
use crate::map::position::Area;

use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input, View};
use crate::view::util::widget_menu::WidgetMenu;
use crate::widget::{Focusable, WidgetType};
use crate::widget::widgets::WidgetList;

pub struct SettingsMenu<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub menu: WidgetMenu
}

impl <'b, B : tui::backend::Backend> View<bool> for SettingsMenu<'_, B>  {
    fn begin(&mut self)  -> Result<InputResult<bool>, Error> {
        // Select the first widget
        if self.menu.widgets.widgets.len() > 0 {
            self.menu.widgets.widgets[0].state_type.focus();
        }

        self.terminal_manager.terminal.clear()?;
        self.draw(None)?;

        while !InputHandler::handle_input(self, None).unwrap().generic_input_result.done {
            self.draw(None)?;
        }
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: true }, view_specific_result: None});
    }

    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let menu_view = &mut self.menu;
        let terminal = &mut self.terminal_manager.terminal;
        let widgets = &menu_view.widgets;
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
}

impl <COM: tui::backend::Backend> InputHandler<bool> for SettingsMenu<'_, COM> {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<bool>, Error> {
        let menu_view = &mut self.menu;
        let key = resolve_input(input);
        let mut target_widget = None;
        match menu_view.widgets.selected_widget {
            Some(idx) => {
                target_widget = Some(&mut menu_view.widgets.widgets[idx as usize]);
            },
            None => {}
        }

        match key {
            Key::Down => {
                menu_view.widgets.next_widget();
            },
            Key::Up => {
                menu_view.widgets.previous_widget();
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
                return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: None});
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
        return Ok(InputResult { generic_input_result: GenericInputResult { done: false, requires_view_refresh: false }, view_specific_result: None});
    }
}