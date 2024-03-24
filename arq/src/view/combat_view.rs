use std::io;
use std::io::Error;
use log::error;
use termion::event::Key;
use tui::layout::{Constraint, Direction, Layout, Rect};



use crate::character::battle::Battle;
use crate::character::equipment::{WeaponSlot};
use crate::map::position::{Area, Position};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{UI};
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input, verify_display_size, View};
use crate::view::framehandler::combat::{CombatFrameHandler};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::util::callback::Callback;
use crate::engine::combat::CombatTurnChoice;
use crate::engine::level::Level;
use crate::ui::ui_areas::{BorderedArea, UI_AREA_NAME_MAIN};
use crate::ui::ui_layout::LayoutType;
use crate::ui::ui_util::{center_area, MIN_AREA};

pub struct CombatView<'a, B : tui::backend::Backend>  {
    ui : &'a mut UI,
    terminal_manager : &'a mut TerminalManager<B>,
    level: Level,
    battle : Battle,
    frame_handler : CombatFrameHandler,
    callback : Box<dyn FnMut(CombatCallbackData) -> Option<CombatCallbackData> + 'a>
}

impl <B: tui::backend::Backend> CombatView<'_, B> {
    pub fn new<'a>(ui: &'a mut UI, terminal_manager: &'a mut TerminalManager<B>, level: Level, battle: Battle) -> CombatView<'a, B> {
        let frame_handler = CombatFrameHandler::new(level.clone());
        let callback = Box::new(|_data| {None});
        CombatView { ui, terminal_manager, level: level, battle, frame_handler, callback }
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
        while self.battle.in_progress {
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

    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let battle = &mut self.battle;
        let _player = battle.characters.get_player_mut();
        let _npcs = battle.characters.get_npcs();

        let ui = &mut self.ui;
        ui.show_console();
        self.terminal_manager.clear_screen();
        verify_display_size::<B>(self.terminal_manager);
        let fh = &mut self.frame_handler;

        let frame_area = Area::from_rect(self.terminal_manager.terminal.get_frame().size());
        let mut ui_layout = ui.ui_layout.as_mut().unwrap();
        let ui_areas = ui_layout.get_or_build_areas(frame_area.to_rect(), LayoutType::COMBAT_VIEW);

        // TODO get the view areas and pass them to the FrameHandler
        self.terminal_manager.terminal.draw(|frame| {
                // TODO using frame_size here is risky and doesn't respect UILayout
                let frame_data = FrameData { frame_area: frame_area, data: battle.clone(), ui_areas: ui_areas.clone() };
                fh.handle_frame(frame, frame_data);
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
                let _option_chosen = selection.options.get(selection.index as usize).unwrap().clone();

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
        if let Some(_data) = data.clone() {
            // TODO pass messages into the framehandler
        }
    }
}

#[cfg(test)]
mod tests {
    use tui::layout::Rect;
    use crate::view::combat_view::build_view_areas;

    #[test]
    fn test_build_view_areas() {
        // GIVEN a frame size of 80x24
        let frame_size = Rect::new(0,0, 80, 24);
        // WHEN we call to build view areas
        let view_areas = build_view_areas(frame_size).unwrap();

        // THEN we expect
        // A main area of
        // the entire width
        // with 80% of the total height
        let main_area = view_areas.main_area;
        assert_eq!(80, main_area.outer.width);
        // With 80% of the 24 lines (24/100 * 80 = 19.2, rounded down = 19)
        assert_eq!(19, main_area.outer.height);

        // A console area of y
        let console_area = view_areas.console_area;
        // With 80% of the total frame width (80/100 * 80 = 64)
        assert_eq!(64, console_area.outer.width);
        // With 20% of the total height of 24 lines (24/100 * 20 = 4.8, rounded up = 5)
        assert_eq!(5, console_area.outer.height);

        // A minimap area
        // With 20% of the total frame width (80/100 * 30 = 16)
        let minimap_area = view_areas.minimap_area;
        assert_eq!(16, minimap_area.outer.width);
        // AND the same height as the console width
        assert_eq!(5, minimap_area.outer.height);

    }
}