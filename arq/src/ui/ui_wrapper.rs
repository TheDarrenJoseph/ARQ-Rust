use std::collections::HashMap;
use std::io;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use crate::character::Character;
use crate::engine::level::{Level, LevelChange};
use crate::map::position::{Area, build_rectangular_area, build_square_area, Position};
use crate::map::room::Room;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::{Draw, get_input_key, UI};
use crate::ui::ui_areas::UIAreas;
use crate::view::framehandler::character::{CharacterFrameHandler, CharacterFrameHandlerInputResult, ViewMode};
use crate::view::{GenericInputResult, InputHandler, InputResult, View};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::character::CharacterFrameHandlerInputResult::VALIDATION;
use crate::view::map_view::MapView;
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
        self.ui.set_console_buffer(message);
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
                    self.ui.set_console_buffer(message);
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

    fn calculate_map_view_area(&self) -> Option<Area> {
        if let Some(frame_size) = self.ui.frame_size.clone() {
            let view_areas = self.ui.get_view_areas(frame_size.to_rect());

            let main_area = view_areas.get_main_area();
            // Main area does not consider borders, so +1 to start inside those
            let map_view_start_pos = Position { x: main_area.x + 1, y: main_area.y + 1 };
            // Build the view area, -1 for remaining border on the other sides
            let map_view_area = build_rectangular_area(map_view_start_pos, main_area.width - 1, main_area.height - 1);
            return Some(map_view_area);
        }
        return None;
    }

    // TODO test this logic
    fn calculate_map_display_area(&self, map_view_area: Area) -> Area {
        // TODO center on the player in future??
        // Setting the display area to the player's position with a -3 offset
        //let mut player_position = level.characters.get_player().unwrap().get_position();
        //let display_area_start = player_position.offset(-3, -3);
        let display_area_start = Position { x: 0, y: 0};

        // The display area is essentially:
        // 1. starting at the map offset of display_area_start
        // 2. ending at display_area_start + the size of the map view area (giving a map co-ord range)
        let map_display_area = build_rectangular_area(display_area_start, map_view_area.size_x - map_view_area.start_position.x , map_view_area.size_y - map_view_area.start_position.y );

        return map_display_area;
    }

    pub(crate) fn draw_map_view(&mut self, level: &mut Level) -> Result<(), io::Error> {
        let map = &mut level.get_map_mut().cloned();
        let frame_size_copy = self.ui.frame_size.clone();
        match map {
            Some(m) => {
                if let Some(frame_size) = frame_size_copy {
                    // Add the UI usage hint to the console buffer
                    self.ui.set_console_buffer(UI_USAGE_HINT.to_string());

                    // There are 3 map areas to consider:
                    // 1. Map Area (the position/size of the actual map e.g tiles), this should currently always start at 0,0
                    // 2. Map view area (The position/size of the map view relative to the entire terminal frame), this could start at 1,1 for example (accounting for borders)
                    // 3. Map display area (The position/size of the map 'viewfinder', the area that you can actually see the map through)
                    // 3.1 The map display area is what will move with the character throughout larger maps
                    let map_view_area = self.calculate_map_view_area();
                    let map_display_area = self.calculate_map_display_area(map_view_area.unwrap());

                    let mut map_view = MapView { map: m, characters: level.characters.clone(), ui: &mut self.ui, terminal_manager: &mut self.terminal_manager, view_area: map_view_area, map_display_area };

                    // Draw the base UI (incl. console) and the map
                    map_view.draw(map_view_area)?;
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