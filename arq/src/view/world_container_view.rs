use std::io;
use std::io::Error;
use tui::layout::{Rect};
use tui::text::{Spans, Span};
use tui::style::{Style, Color, Modifier};
use tui::symbols::line::VERTICAL;
use tui::buffer::{Buffer};
use tui::widgets::{Block, Borders};
use termion::input::TermRead;
use termion::event::Key;
use std::slice::Iter;
use std::collections::HashMap;

use crate::ui::{UI, FrameHandler, FrameData};
use crate::view::{View, resolve_input, InputResult, GenericInputResult};
use crate::view::framehandler::container_view;
use crate::terminal::terminal_manager::TerminalManager;
use crate::character::{get_all_attributes, Character, Race, Class, determine_class, Attribute};
use crate::widget::text_widget::build_text_input;
use crate::widget::dropdown_widget::{build_dropdown, DropdownInputState};
use crate::widget::number_widget::{build_number_input, build_number_input_with_value, NumberInputState};
use crate::widget::button_widget::build_button;
use crate::widget::character_stat_line::{build_character_stat_line, CharacterStatLineState};
use crate::widget::{Focusable, Widget, WidgetType, Named};
use crate::view::framehandler::character_view::{CharacterView, ViewMode};
use crate::view::framehandler::container_view::{ContainerView, build_container_view, ContainerViewInputResult};
use crate::map::position::Area;
use crate::view::InputHandler;
use crate::map::objects::container::Container;
use crate::view;
use crate::view::callback::Callback;

// Combines multiple character info views into one w/ tabbing
pub struct WorldContainerView<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: WorldContainerViewFrameHandler,
    pub container : Container,
    pub callbacks : HashMap<String, Box<dyn FnMut(ContainerViewInputResult) + 'a>>
}

pub struct WorldContainerViewFrameData {
}

pub struct WorldContainerViewFrameHandler {
    pub container_views : Vec<ContainerView>
}

impl <B : tui::backend::Backend> WorldContainerView<'_, B> {
}

impl <B : tui::backend::Backend> View<'_, ContainerViewInputResult> for WorldContainerView<'_, B>  {
    fn begin(&mut self)  -> Result<bool, Error> {
        let view = container_view::build_container_view( self.container.clone());
        self.frame_handler.container_views = vec!(view);
        self.terminal_manager.terminal.clear();
        self.draw(None);
        while !self.handle_input(None).unwrap() {
            self.draw(None);
        }
        Ok(true)
    }


    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        let ui = &mut self.ui;

        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
            let areas = ui.get_view_areas(frame.size());
            let view_area = areas[0];
            let frame_area = Rect { x : view_area.x + 1, y : view_area.y + 1, width: view_area.width.clone() - 2,  height: view_area.height.clone() - 2};
            let specific_frame_data = WorldContainerViewFrameData { };
            frame_handler.handle_frame(frame, FrameData { frame_size: frame_area, data: specific_frame_data });
        })?;
        Ok(())
    }

    fn handle_input(&mut self, input: Option<Key>) -> Result<bool, Error> {
        let key = resolve_input(input);
        match key {
            Key::Char('q') => {
                // Drop the last container view and keep going
                let mut container_views = &mut self.frame_handler.container_views;
                if container_views.len() > 1 {
                    if let Some(closing_view) = self.frame_handler.container_views.pop() {
                        let closing_container = closing_view.container;
                        if let Some(parent_view) = self.frame_handler.container_views.last_mut() {
                            let parent_container = &mut parent_view.container;
                            if let Some(position) = parent_container.position(&closing_container) {
                                parent_container.replace(position, closing_container);
                            }
                        }
                    }
                    return Ok(false)
                } else if container_views.len() == 1 {
                    let last_view = &mut self.frame_handler.container_views[0];
                    self.container = last_view.container.clone();
                }
                return Ok(true)
            },
            Key::Char('t') => {
                if let Some(parent_view) = self.frame_handler.container_views.last_mut() {
                    let selected_items = parent_view.get_selected_items();

                    let mut to_remove = Vec::new();
                    for item in selected_items {
                        if let Some(found) = parent_view.container.find(&item) {
                            to_remove.push(found.clone());
                        }
                    }
                    let mut view_container = &mut parent_view.container;
                    view_container.remove_matching_items(to_remove);
                    let selected_container_items = parent_view.get_selected_items();
                    parent_view.reset_selection();
                    let result = ContainerViewInputResult::TAKE_ITEMS(selected_container_items);
                    self.trigger_callback(String::from("t"), result);
                }
            },
            // Passthrough anything not handled here into the sub framehandler
            _ => {
                let mut generic_input_result : Option<GenericInputResult> = None;
                let mut container_views = &mut self.frame_handler.container_views;
                let have_container_views = !container_views.is_empty();
                if have_container_views {
                    if let Some(topmost_view) = container_views.last_mut() {
                        let mut container_view_input_result = topmost_view.handle_input(Some(key));
                        let result = container_view_input_result.unwrap();
                        if let Some(ContainerViewInputResult::OPEN_CONTAINER_VIEW(stacked_view)) = result.view_specific_result {
                            container_views.push(stacked_view);
                        }
                        generic_input_result = Some(result.generic_input_result);
                    }
                }

                if let Some(r) = generic_input_result {
                    if r.requires_view_refresh {
                        self.terminal_manager.terminal.clear();
                    }
                }
            }
        }

        return Ok(false)
    }
}

impl <'c, B : tui::backend::Backend> Callback<'c, ContainerViewInputResult> for WorldContainerView<'c, B> {
    fn set_callback(&mut self, event_name: String, mut callback: Box<impl FnMut(ContainerViewInputResult) + 'c>) {
        self.callbacks.insert(event_name, callback);
    }

    fn trigger_callback(&mut self, event_name: String, data: ContainerViewInputResult) {
        if self.callbacks.contains_key(&event_name) {
            let mut cb = self.callbacks.get_mut(&event_name).unwrap();
            cb(data);
        }
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, WorldContainerViewFrameData> for WorldContainerViewFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<WorldContainerViewFrameData>) {
        if let Some(topmost_view) = self.container_views.last_mut() {
            let mut frame_inventory = topmost_view.container.clone();
            let frame_size = frame.size();
            topmost_view.handle_frame(frame, FrameData { frame_size: data.frame_size, data: &mut frame_inventory });
        }
    }
}