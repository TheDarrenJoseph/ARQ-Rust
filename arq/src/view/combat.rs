use std::io::Error;
use termion::event::Key;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders};
use crate::character::battle::Battle;
use crate::map::position::{Area, build_rectangular_area, Position, start_position_from_rect};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input, View};
use crate::view::framehandler::combat::CombatFrameHandler;
use crate::view::framehandler::{FrameData, FrameHandler};

pub struct CombatView<'a, B : tui::backend::Backend>  {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub battle : Battle,
    pub frame_handler : CombatFrameHandler
}

impl <B : tui::backend::Backend> CombatView<'_, B> {
    fn build_done_result(&self) -> InputResult<Battle> {
        InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: Some(self.battle.clone())}
    }

    fn build_input_done_result(&self) -> InputResult<bool> {
        InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: false }, view_specific_result: None}
    }

    fn build_input_not_done_result(&self) -> InputResult<bool> {
        InputResult { generic_input_result: GenericInputResult { done: false, requires_view_refresh: false }, view_specific_result: None}
    }
}

impl <B : tui::backend::Backend> View<Battle> for CombatView<'_, B>  {
    fn begin(&mut self) -> Result<InputResult<Battle>, Error> {
        // Input / Output loop
        while(self.battle.in_progress) {
            self.draw(None);

            let input_result = self.handle_input(None).unwrap();
            if input_result.generic_input_result.done {
                return Ok(self.build_done_result());
            }

            // TODO get battle action
            // callback w/ action to trigger processing
            // Show action results
        }
        return Ok(self.build_done_result());
    }

    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
        let battle = &mut self.battle;
        let player = battle.characters.get_player_mut();
        let npcs = battle.characters.get_npcs();

        let ui = &mut self.ui;
        ui.show_console();
        self.terminal_manager.clear_screen();
        let fh = &mut self.frame_handler;
        self.terminal_manager.terminal.draw(|frame| {
            let ui_areas = ui.get_view_areas(frame.size());
            fh.areas = Some(ui_areas);
            let frame_data = FrameData { frame_size: frame.size(), data: battle.clone() };
            fh.handle_frame(frame, frame_data);
        });

        return Ok(());
    }
}

impl <COM: tui::backend::Backend> InputHandler<bool> for CombatView<'_, COM> {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<bool>, Error> {
        let key = resolve_input(input)?;
        match key {
            Key::Esc => {
                return Ok(self.build_input_done_result());
            },
            _ => {
                return Ok(self.build_input_not_done_result());
            }
        }
    }
}
