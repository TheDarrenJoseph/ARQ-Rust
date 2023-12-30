use std::io::Error;
use log::error;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Paragraph};

use crate::map::position::Area;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{get_input_key, UI};
use crate::ui::ui_util::{center_area, MIN_AREA};
use crate::view::{GenericInputResult, InputResult, View};

pub struct DialogView<'a, B : tui::backend::Backend> {
    message: String,
    ui : &'a mut UI,
    terminal_manager : &'a mut TerminalManager<B>,
}

impl <B : tui::backend::Backend> DialogView<'_, B> {
    pub fn new<'a>(ui: &'a mut UI, terminal_manager: &'a mut TerminalManager<B>, message: String) -> DialogView<'a, B> {
        DialogView { ui, terminal_manager, message }
    }
}

impl <'b, B : tui::backend::Backend> View<()> for DialogView<'_, B>  {
    fn begin(&mut self) -> Result<InputResult<()>, Error> {
        self.draw(None);
        get_input_key();
        Ok(InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        })
    }

    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let message = self.message.clone();
        let _ui = &mut self.ui;
        self.terminal_manager.clear_screen();
        self.terminal_manager.terminal.draw(|frame| {

            // First check for the minimum space and center the dialog
            let centered_area_result = center_area(MIN_AREA, frame.size(), MIN_AREA);
            if let Ok(area) = centered_area_result {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black));
                frame.render_widget(block, area);

                let paragraph = Paragraph::new(Span::from(message.clone()));
                let message_area = Rect::new(area.x + 2,area.y + 2, frame.size().width / 2, 1);
                frame.render_widget(paragraph, message_area);

                let enter_text = String::from("[Enter]");
                let paragraph = Paragraph::new(Span::from(enter_text.clone()));
                let message_area = Rect::new((area.width + area.x) / 2 - enter_text.len() as u16, area.height + area.y - 1, enter_text.len() as u16, 1);
                frame.render_widget(paragraph, message_area);
            }  else {
                let err = centered_area_result.err().unwrap();
                error!("{}", err);
                // TODO update views to be able to return Error
            }
        });
        Ok(())
    }
}
