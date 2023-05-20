use std::collections::HashMap;
use std::io;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use crate::character::Character;
use crate::engine::level::{Level, LevelChange};
use crate::map::position::{build_rectangular_area, Position};
use crate::map::room::Room;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{Draw, get_input_key, UI};
use crate::ui::ui_areas::UIAreas;
use crate::view::framehandler::character::{CharacterFrameHandler, CharacterFrameHandlerInputResult, ViewMode};
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::character::CharacterFrameHandlerInputResult::VALIDATION;
use crate::view::map::MapView;
use crate::view::model::usage_line::{UsageCommand, UsageLine};
use crate::widget::widgets::WidgetList;

pub struct UIWrapper<B: 'static + tui::backend::Backend> {
    pub(crate) ui : UI,
    pub(crate) terminal_manager : TerminalManager<B>,
}

const UI_USAGE_HINT: &str = "Use the arrow keys/WASD to move.\nEsc - Menu";

impl <B : Backend> UIWrapper<B> {
    // TODO refactor into a singular component shared with commands
    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    pub(crate) fn print_and_re_render(&mut self, message: String) -> Result<(), io::Error> {
        self.ui.console_print(message);
        self.re_render()
    }

    pub fn get_prompted_input(&mut self, prompt: String) -> Result<Key, io::Error> {
        self.print_and_re_render(prompt)?;
        get_input_key()
    }

    pub fn yes_or_no(&mut self, prompt: String, confirm_message: Option<String>) -> Result<bool, io::Error> {
        let final_prompt = format!("{} (y/n)", prompt);
        loop {
            match self.get_prompted_input(final_prompt.clone())? {
                Key::Char('y') | Key::Char('Y') => {
                    if let Some(message) = confirm_message {
                        let final_message = format!("{} (any key to continue)", message);
                        self.print_and_re_render(final_message);
                        get_input_key();
                    }
                    return Ok(true);
                },
                Key::Char('n') | Key::Char('N')  => {
                    return Ok(false);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    pub(crate) fn draw_start_menu(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_start_menu(frame) })
    }

    pub(crate) fn draw_info(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| { ui.draw_info(frame) })
    }

    // TODO this should live in it's own view likely
    // Shows character creation screen
    // Returns the finished character once input is confirmed
    fn show_character_creation(&mut self, base_character: Character) -> Result<Character, io::Error> {
        let mut character_view = CharacterFrameHandler { character: base_character.clone(),  widgets: WidgetList { widgets: Vec::new(), widget_index: None }, view_mode: ViewMode::CREATION, attributes_area: Rect::new(0, 0, 0, 0)};
        // Begin capture of a new character
        let mut character_creation_result = InputResult { generic_input_result:
        GenericInputResult { done: false, requires_view_refresh: false },
            view_specific_result: None
        };
        while !character_creation_result.generic_input_result.done {
            let ui = &mut self.ui;
            ui.show_console();
            self.terminal_manager.terminal.draw(|frame| {
                let areas: UIAreas = ui.get_view_areas(frame.size());
                let mut main_area = areas.get_main_area();
                main_area.height -= 2;
                ui.render(frame);
                character_view.handle_frame(frame, FrameData { data: base_character.clone(), frame_size: main_area });
            })?;
            ui.hide_console();

            let key = get_input_key()?;
            character_creation_result = character_view.handle_input(Some(key))?;

            match character_creation_result.view_specific_result {
                Some(VALIDATION(message)) => {
                    self.ui.console_print(message);
                    self.re_render()?;
                },
                Some(CharacterFrameHandlerInputResult::NONE) => {
                    return Ok(character_view.get_character())
                },
                _ => {}
            }
        }
        return Ok(character_view.get_character());
    }

    pub(crate) fn draw_map_view(&mut self, level: &mut Level) -> Result<(), io::Error> {
        let map = &mut level.get_map_mut().cloned();
        let frame_size_copy = self.ui.frame_size.clone();
        match map {
            Some(m) => {
                if let Some(frame_size) = frame_size_copy {

                    let mut map_view = MapView { map: m, characters: level.characters.clone(), ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, view_area: None };

                    // Adjust the map view size to fit within our borders / make space for the console
                    let map_view_start_pos = Position { x : frame_size.start_position.x + 1, y: frame_size.start_position.y + 1};
                    let map_view_frame_size = Some(build_rectangular_area(map_view_start_pos, frame_size.size_x - 2, frame_size.size_y - 8 ));
                    map_view.draw(map_view_frame_size)?;
                    map_view.draw_containers()?;
                    map_view.draw_characters()?;
                    self.ui.console_print(UI_USAGE_HINT.to_string());
                    self.re_render()?;
                }
            },
            None => {}
        }
        Ok(())
    }

    pub fn check_room_entry_exits(&mut self, room: &Room, pos: Position) -> LevelChange {
        if pos.equals_option(room.get_exit()) {
            match self.yes_or_no(
                String::from("You've reached the exit! There's a staircase downwards; would you like to leave?"),
                Some(String::from("You move downstairs a level.."))) {
                Ok(true) => {
                    return LevelChange::DOWN;
                },
                _ => {}
            }
        } else if pos.equals_option(room.get_entry()) {
            match self.yes_or_no(
                String::from("This is the entrance. There's a staircase upwards; wold you like to leave?"),
                Some(String::from("You move upstairs a level.."))) {
                Ok(true) => {
                    return LevelChange::UP;
                },
                _ => {}
            }
        }

        return LevelChange::NONE;
    }

    pub fn clear_screen(&mut self) -> Result<(), io::Error> {
        self.terminal_manager.terminal.clear()
    }
}