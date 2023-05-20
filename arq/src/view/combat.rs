use std::io;
use std::io::Error;
use log::error;
use termion::event::Key;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders};
use crate::character::battle::Battle;
use crate::character::equipment::{EquipmentSlot, WeaponSlot};
use crate::map::position::{Area, build_rectangular_area, Position, start_position_from_rect};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{get_input_key, UI};
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input, View};
use crate::view::framehandler::combat::CombatFrameHandler;
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::util::callback::Callback;
use crate::engine::combat::CombatTurnChoice;
use crate::ui::ui_util::{center_area, MIN_AREA};

pub struct CombatView<'a, B : tui::backend::Backend>  {
    ui : &'a mut UI,
    terminal_manager : &'a mut TerminalManager<B>,
    battle : Battle,
    frame_handler : CombatFrameHandler,
    callback : Box<dyn FnMut(CombatCallbackData) -> Option<CombatCallbackData> + 'a>
}

impl <B: tui::backend::Backend> CombatView<'_, B> {
    pub fn new<'a>(ui: &'a mut UI, terminal_manager: &'a mut TerminalManager<B>, battle: Battle) -> CombatView<'a, B> {
        let frame_handler = CombatFrameHandler::new();
        let callback = Box::new(|_data| {None});
        CombatView { ui, terminal_manager, battle, frame_handler, callback }
    }

    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }
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
            let frame_size = frame.size();
            let centered = center_area(MIN_AREA, frame_size, MIN_AREA);

            if (centered.is_ok()) {
                let ui_areas = ui.get_view_areas(centered.unwrap());
                fh.areas = Some(ui_areas);
                let frame_data = FrameData { frame_size: frame.size(), data: battle.clone() };
                fh.handle_frame(frame, frame_data);
            } else {
                let err = centered.err().unwrap();
                error!("{}", err)
            }
        });

        return Ok(());
    }
}

impl <COM: tui::backend::Backend> InputHandler<bool> for CombatView<'_, COM> {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<bool>, Error> {
        let key = resolve_input(input)?;
        match key {
            Key::Up => {
                let index = self.frame_handler.selection.index;
                let option_count = self.frame_handler.selection.options.len();
                if option_count > 0 && index > 0 {
                    self.frame_handler.selection.index -= 1;
                }
                return Ok(self.build_input_not_done_result());
            },
            Key::Down => {
                let index = self.frame_handler.selection.index;
                let option_count = self.frame_handler.selection.options.len();
                if option_count > 0 && index < option_count as u16 - 1 {
                    self.frame_handler.selection.index += 1;
                }
                return Ok(self.build_input_not_done_result());
            },
            // Enter key
            Key::Char('\n') => {
                let selection = &self.frame_handler.selection;
                let option_chosen = selection.options.get(selection.index as usize).unwrap().clone();

                let data = CombatCallbackData { choice: CombatTurnChoice::ATTACK(WeaponSlot::PRIMARY), result: None };
                self.trigger_callback(data);

                // TODO send-recieve battle turn option
                return Ok(self.build_input_not_done_result());
            },
            Key::Esc => {
                return Ok(self.build_input_done_result());
            },
            _ => {
                return Ok(self.build_input_not_done_result());
            }
        }
    }
}

#[derive(Clone)]
pub struct CombatCallbackData {
    pub choice: CombatTurnChoice,
    pub result: Option<CombatResult>
}

#[derive(Clone)]
pub struct CombatResult {
    pub(crate) messages: Vec<String>
}

impl <'a, B : tui::backend::Backend> Callback <'a, CombatCallbackData> for CombatView<'a, B>  {
    fn set_callback(&mut self, callback: Box<impl FnMut(CombatCallbackData) -> Option<CombatCallbackData> + 'a>) {
        self.callback = callback;
    }

    /*
        Triggers a callback to handle the battle logic behind a given combat turn choice
     */
    fn trigger_callback(&mut self, data: CombatCallbackData) {
        let result = (self.callback)(data);
        self.handle_callback_result(result);
    }

    /*
        Any information about the result of a battle action callback will be handled here
     */
    fn handle_callback_result(&mut self, data: Option<CombatCallbackData>) {
        let result = data.unwrap().result.unwrap();
        for message in result.messages {
            // TODO either print and re-render here or in the frame handler
            // self.ui.console_print(message);
            // let keyc = get_input_key();
        }
    }
}
