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

// Combines multiple character info views into one w/ tabbing
pub struct WorldContainerView<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: WorldContainerViewFrameHandler,
    pub container : Container
}

pub struct WorldContainerViewFrameData {
}

pub struct WorldContainerViewFrameHandler {
    pub container_views : Vec<ContainerView>
}

impl <B : tui::backend::Backend> WorldContainerView<'_, B> {
}

impl <B : tui::backend::Backend> View for WorldContainerView<'_, B>  {
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

        let mut frame_area = Rect::default();
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
            let size = frame.size();
            frame_area = Rect { x : size.x.clone() + 1, y : size.y.clone() + 2, width: size.width.clone() -2,  height: size.height.clone() - 2};

            let specific_frame_data = WorldContainerViewFrameData { };
            frame_handler.handle_frame(frame, FrameData { frame_size: frame.size(), data: specific_frame_data });
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

impl <B : tui::backend::Backend> FrameHandler<B, WorldContainerViewFrameData> for WorldContainerViewFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<WorldContainerViewFrameData>) {
        if let Some(topmost_view) = self.container_views.last_mut() {
            let mut frame_inventory = topmost_view.container.clone();
            let frame_size = frame.size();
            let inventory_area = Rect::new(3, 3, frame_size.width - 6, frame_size.height - 3);
            topmost_view.handle_frame(frame, FrameData { frame_size: inventory_area, data: &mut frame_inventory });
        }
    }
}