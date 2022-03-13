use std::io::Error;
use tui::layout::{Rect};
use termion::event::Key;
use std::collections::HashMap;

use crate::ui::{UI, FrameHandler, FrameData};
use crate::view::{View, resolve_input, GenericInputResult};
use crate::view::framehandler::container;
use crate::terminal::terminal_manager::TerminalManager;
use crate::view::framehandler::container::{ContainerFrameHandler, ContainerFrameHandlerInputResult};
use crate::map::position::Area;
use crate::view::InputHandler;
use crate::map::objects::container::Container;
use crate::view::callback::Callback;

// Combines multiple character info views into one w/ tabbing
pub struct WorldContainerView<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handlers: WorldContainerViewFrameHandlers,
    pub container : Container,
    pub callback : Box<dyn FnMut(ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> + 'a>
}

pub struct WorldContainerViewFrameData {
}

pub struct WorldContainerViewFrameHandlers {
    pub frame_handlers: Vec<ContainerFrameHandler>
}

impl <B : tui::backend::Backend> WorldContainerView<'_, B> {
    fn clone_selected_container_items(&mut self) -> Vec<Container> {
        let mut items = Vec::new();
        if let Some(parent_view) = self.frame_handlers.frame_handlers.last_mut() {
            let selected_items = parent_view.get_selected_items();
            for item in selected_items {
                if let Some(found) = parent_view.container.find(&item) {
                    items.push(found.clone());
                }
            }
        }
        items
    }

    fn handle_callback_result(&mut self, result: Option<ContainerFrameHandlerInputResult>) {
        if let Some(r) = result {
            if let Some(topmost_view) = self.frame_handlers.frame_handlers.last_mut() {
                topmost_view.handle_callback_result(r);
            }
        }
    }
}

impl <B : tui::backend::Backend> View<'_, ContainerFrameHandlerInputResult> for WorldContainerView<'_, B>  {
    fn begin(&mut self)  -> Result<bool, Error> {
        self.terminal_manager.terminal.clear();
        self.draw(None);
        while !self.handle_input(None).unwrap() {
            self.draw(None);
        }
        Ok(true)
    }


    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handlers;
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
            Key::Char('t') => {
                if let Some(parent_view) = self.frame_handlers.frame_handlers.last_mut() {
                    let selected_container_items = parent_view.get_selected_items();
                    let result = ContainerFrameHandlerInputResult::TAKE_ITEMS(selected_container_items);
                    self.trigger_callback(result);
                }
            },
            Key::Char('q') => {
                // Drop the last container view and keep going
                let container_views = &self.frame_handlers.frame_handlers;
                if container_views.len() > 1 {
                    if let Some(closing_view) = self.frame_handlers.frame_handlers.pop() {
                        let closing_container = closing_view.container;
                        if let Some(parent_view) = self.frame_handlers.frame_handlers.last_mut() {
                            let parent_container = &mut parent_view.container;
                            if let Some(position) = parent_container.position(&closing_container) {
                                parent_container.replace(position, closing_container);
                            }
                        }
                    }
                    return Ok(false)
                } else if container_views.len() == 1 {
                    let last_view = &mut self.frame_handlers.frame_handlers[0];
                    self.container = last_view.container.clone();
                }
                return Ok(true)
            },
            // Passthrough anything not handled here into the sub framehandler
            _ => {
                let mut generic_input_result : Option<GenericInputResult> = None;
                let container_views = &mut self.frame_handlers.frame_handlers;
                let have_container_views = !container_views.is_empty();
                if have_container_views {
                    if let Some(topmost_view) = container_views.last_mut() {
                        let container_view_input_result = topmost_view.handle_input(Some(key));
                        let result = container_view_input_result.unwrap();
                        if let Some(ContainerFrameHandlerInputResult::OPEN_CONTAINER_VIEW(stacked_view)) = result.view_specific_result {
                            container_views.push(stacked_view);
                        } else if let Some(r) = result.view_specific_result {
                            self.trigger_callback(r);
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

impl <'c, B : tui::backend::Backend> Callback<'c, ContainerFrameHandlerInputResult> for WorldContainerView<'c, B> {
    fn set_callback(&mut self, callback: Box<impl FnMut(ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> + 'c>) {
        self.callback = callback;
    }

    fn trigger_callback(&mut self, data: ContainerFrameHandlerInputResult) {
        let result = (self.callback)(data);
        self.handle_callback_result(result);
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, WorldContainerViewFrameData> for WorldContainerViewFrameHandlers {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<WorldContainerViewFrameData>) {
        if let Some(topmost_view) = self.frame_handlers.last_mut() {
            let mut frame_inventory = topmost_view.container.clone();
            topmost_view.handle_frame(frame, FrameData { frame_size: data.frame_size, data: &mut frame_inventory });
        }
    }
}