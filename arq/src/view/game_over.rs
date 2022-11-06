use std::io::Error;
use termion::event::Key;
use tui::layout::{Alignment, Rect};
use tui::style::Style;
use tui::text::Span;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use crate::map::position::Area;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input, View};
use crate::view::game_over::GameOverChoice::{EXIT, RESTART};
use crate::widget::button_widget::build_button;
use crate::widget::widgets::WidgetList;
use crate::widget::{Focusable, WidgetType};
use crate::widget::WidgetType::Button;

pub struct GameOver<'a, B : tui::backend::Backend> {
    pub message : String,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub widgets: WidgetList
}

pub enum GameOverChoice {
    RESTART,
    EXIT
}

pub fn build_game_over_menu<'a, B : tui::backend::Backend>(message: String, ui: &'a mut UI, terminal_manager: &'a mut TerminalManager<B>) -> GameOver<'a, B> {
    GameOver { message, ui, terminal_manager, widgets:
    WidgetList { selected_widget: Some(0), widgets :
        vec![ build_button(7, String::from("Restart")),
              build_button(7, String::from("Exit"))
        ]
    }
    }
}

impl <'b, B : tui::backend::Backend> View<GameOverChoice> for GameOver<'_, B>  {
    fn begin(&mut self)  -> Result<InputResult<GameOverChoice>, Error> {
        // Select the first widget
        if self.widgets.widgets.len() > 0 {
            self.widgets.widgets[0].state_type.focus();
        }

        self.terminal_manager.terminal.clear()?;
        self.draw(None)?;

        let mut input_result = self.handle_input(None)?;
        while !input_result.generic_input_result.done {
            input_result = self.handle_input(None)?;
            self.draw(None)?;
        }
        return Ok(input_result);
    }

    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let paragraph = Paragraph::new(self.message.clone())
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default()).alignment(Alignment::Center).wrap(Wrap { trim: true });

        let terminal = &mut self.terminal_manager.terminal;
        let widgets = &self.widgets;
        terminal.draw(|frame| {
            let frame_size = frame.size();
            let mut half_width = frame_size.width.clone() / 2;

            // Remove length of longest title
            if half_width >= 7 {
                half_width -= 7;
            }

            let mut half_height = frame_size.height.clone() / 2;
            let mut offset = 0;

            let frame_size = frame.size();
            let paragraph_size = Rect::new(0, frame_size.height / 4, frame_size.width, 2);
            frame.render_widget(paragraph, paragraph_size);

            for widget in widgets.widgets.iter() {
                let widget_area = Rect::new(half_width, half_height + offset.clone(), frame_size.width.clone() / 2, 1);
                match &widget.state_type {
                    WidgetType::Button(w) => {
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

impl <B : tui::backend::Backend> InputHandler<GameOverChoice> for GameOver<'_, B> {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<GameOverChoice>, Error> {
        let mut target_widget = None;
        match self.widgets.selected_widget {
            Some(idx) => {
                target_widget = Some(&mut self.widgets.widgets[idx as usize]);
            },
            None => {}
        }

        let key = resolve_input(input)?;
        match key {
            Key::Down => {
                self.widgets.next_widget();
            },
            Key::Up => {
                self.widgets.previous_widget();
            },
            Key::Char('\n') => {
                match target_widget {
                    Some(mut widget) => {
                        match &mut widget.state_type {
                            WidgetType::Button(button) => {
                                log::info!("Current widget: {}", button.get_name());
                                // TODO have internal result / name mappings
                                if button.get_name() == "Exit" {
                                    return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: Some(EXIT)});
                                } else if button.get_name() == "Restart" {
                                    return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: Some(RESTART)});
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        return Ok(InputResult { generic_input_result: GenericInputResult { done: false, requires_view_refresh: false }, view_specific_result: None});
    }
}