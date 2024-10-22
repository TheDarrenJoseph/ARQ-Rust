use crate::error::errors::ErrorWrapper;
use crate::input::KeyInputResolver;
use termion::event::Key;
use tui::terminal::CompletedFrame;

use crate::map::objects::container::Container;
use crate::map::position::{Area, Position};
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::ui::ui_areas::{UIAreas, UI_AREA_NAME_MAIN};
use crate::ui::ui_layout::LayoutType;
use crate::view::framehandler::container::{ContainerFrameHandler, ContainerFrameHandlerInputResult, MoveToContainerChoiceData, TakeItemsData};
use crate::view::framehandler::container_choice::{ContainerChoiceFrameHandler, ContainerChoiceFrameHandlerInputResult};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::util::callback::Callback;
use crate::view::InputHandler;
use crate::view::{GenericInputResult, InputResult, View};

/*
    This View is responsible for displaying/interacting with containers in the world (i.e chests, dropped items, dead bodies)
 */
pub struct WorldContainerView<'a, B : tui::backend::Backend> {
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handlers: WorldContainerViewFrameHandlers,
    pub container : Container,
    pub callback : Box<dyn FnMut(ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> + 'a>,
    pub input_resolver: Box<dyn KeyInputResolver>
}

pub struct WorldContainerViewFrameData {
}

pub struct WorldContainerViewFrameHandlers {
    pub container_frame_handlers: Vec<ContainerFrameHandler>,
    pub choice_frame_handler: Option<ContainerChoiceFrameHandler>
}

impl <B : tui::backend::Backend> WorldContainerView<'_, B> {

    /*
    * TODO (DUPLICATION) make choice and generic input flows generic / shared between views i.e world_container and character_info?
    * Handles passthrough to the relevant container choice view (if one is present)
    * Triggering callbacks if needed
    * Returns the optional input result of the container choice view, and a boolean to indicate success
    */
    fn handle_container_choice_input(&mut self, key: Key) -> Result<(Option<GenericInputResult>, bool), ErrorWrapper> {
        if let Some(cfh) = &mut self.frame_handlers.choice_frame_handler {
            let result = cfh.handle_input(Some(key))?;
            if let Some(view_specific_result) = result.view_specific_result {
                match view_specific_result {
                    ContainerChoiceFrameHandlerInputResult::Select(selected_container) => {
                        let container_views = &mut self.frame_handlers.container_frame_handlers;
                        if let Some(topmost_view) = container_views.last_mut() {
                            let view_specific_result = topmost_view.build_move_items_result().unwrap().view_specific_result.unwrap();
                            match view_specific_result {
                                ContainerFrameHandlerInputResult::MoveToContainerChoice(mut data) => {
                                    // Add the target selected to the callback data
                                    data.target_container = Some(selected_container.clone());
                                    self.trigger_callback(ContainerFrameHandlerInputResult::MoveToContainerChoice(data));
                                    // Clear the frame handler now we're done
                                    self.frame_handlers.choice_frame_handler = None;
                                    return Ok((None, true));
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
            }
            return Ok((Some(result.generic_input_result), true))
        }
        Ok((None, false))
    }

}

impl <B : tui::backend::Backend> View<bool> for WorldContainerView<'_, B>  {
    fn begin(&mut self)  -> Result<InputResult<bool>, ErrorWrapper> {
        self.terminal_manager.terminal.clear()?;
        self.draw(None)?;
        
        while !self.handle_input(None).unwrap().generic_input_result.done {
            self.draw(None)?;
        }
        return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: true }, view_specific_result: None});
    }

    fn draw(&mut self, _area: Option<Area>) -> Result<CompletedFrame, ErrorWrapper> {
        let frame_handler = &mut self.frame_handlers;
        let ui = &mut self.ui;

        let ui_layout = ui.ui_layout.as_mut().unwrap();
        let frame_size = self.terminal_manager.terminal.get_frame().size();
        let ui_areas: UIAreas = ui_layout.get_or_build_areas(frame_size, LayoutType::StandardSplit).clone();

        if let Some(main) = ui_areas.get_area(UI_AREA_NAME_MAIN) {
            let main_area = main.area;
            return Ok(self.terminal_manager.terminal.draw(|frame| {
                ui.render(frame);
                let frame_area = Area::new(
                    Position::new(main_area.start_position.x + 1, main_area.start_position.y + 1),
                    main_area.width.clone() - 1,
                    main_area.height.clone() - 2
                );
                let specific_frame_data = WorldContainerViewFrameData {};
                frame_handler.handle_frame(frame, FrameData { frame_area: frame_area, data: specific_frame_data, ui_areas: ui_areas.clone() });
            })?);
        }
        ErrorWrapper::internal_result(String::from("Failed to draw world container view"))
    }
}

impl <COM: tui::backend::Backend> InputHandler<bool> for WorldContainerView<'_, COM> {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<bool>, ErrorWrapper> {
        let key = self.input_resolver.get_or_return_input_key(input)?;
        match key {
            Key::Char('t') => {
                if let Some(parent_view) = self.frame_handlers.container_frame_handlers.last_mut() {
                    let selected_container_items = parent_view.get_selected_items();
                    let data = TakeItemsData { source: self.container.clone(), to_take: selected_container_items, position: None };
                    let result = ContainerFrameHandlerInputResult::TakeItems(data);
                    self.trigger_callback(result);
                }
            },
            Key::Esc => {
                // Drop the last container view and keep going
                let container_views = &self.frame_handlers.container_frame_handlers;
                if container_views.len() > 1 {
                    if let Some(closing_view) = self.frame_handlers.container_frame_handlers.pop() {
                        let closing_container = closing_view.container;
                        if let Some(parent_view) = self.frame_handlers.container_frame_handlers.last_mut() {
                            let parent_container = &mut parent_view.container;
                            if let Some(position) = parent_container.position(&closing_container) {
                                parent_container.replace(position, closing_container);
                            }
                        }
                    }
                    return Ok(InputResult { generic_input_result: GenericInputResult { done: false, requires_view_refresh: false }, view_specific_result: None});
                } else if container_views.len() == 1 {
                    let last_view = &mut self.frame_handlers.container_frame_handlers[0];
                    self.container = last_view.container.clone();
                }
                return Ok(InputResult { generic_input_result: GenericInputResult { done: true, requires_view_refresh: true }, view_specific_result: None});
            },
            // Passthrough anything not handled here into the sub framehandler
            _ => {
                let mut generic_input_result: Option<GenericInputResult> = None;

                // TODO (DUPLICATE) make generic between world/inventory view?
                // Container choice view takes precedence as it's basically a dialog
                // Container choice handlers take priority as they're essentially a dialog
                if let Ok((gir_result, success)) = self.handle_container_choice_input(key) {
                    // Rewrap this only if something was returned
                    if let Some(gir) = gir_result {
                        generic_input_result = Some(gir);
                    }
                    // Force a redraw
                    if success {
                        return Ok(InputResult { generic_input_result: GenericInputResult { done: false, requires_view_refresh: true }, view_specific_result: None});
                    }
                }

                let container_views = &mut self.frame_handlers.container_frame_handlers;
                let have_container_views = !container_views.is_empty();
                if have_container_views {
                    if let Some(topmost_view) = container_views.last_mut() {
                        let view_input_result = topmost_view.handle_input(Some(key));
                        let result = view_input_result.unwrap();
                        if let Some(ContainerFrameHandlerInputResult::OpenContainerView(stacked_view)) = result.view_specific_result {
                            container_views.push(stacked_view);
                        } else if let Some(r) = result.view_specific_result {
                            self.trigger_callback(r);
                        }
                        generic_input_result = Some(result.generic_input_result);
                    }
                }

                if let Some(r) = generic_input_result {
                    if r.requires_view_refresh {
                        self.terminal_manager.terminal.clear()?;
                    }
                }
            }
        }

        return Ok(InputResult { generic_input_result: GenericInputResult { done: false, requires_view_refresh: false }, view_specific_result: None});
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

    fn handle_callback_result(&mut self, result: Option<ContainerFrameHandlerInputResult>) {
        if let Some(r) = result {
            match r {
                ContainerFrameHandlerInputResult::MoveToContainerChoice(ref data) => {
                    self.frame_handlers.build_container_choice_view(data);
                },
                ContainerFrameHandlerInputResult::MoveItems(ref data) => {
                    // TODO (Duplicate) make generic between world_contaienr / character_info
                    if data.target_container.is_some() {
                        // if target_container is the root view container
                        let root_container = self.frame_handlers.container_frame_handlers.first().map(|top_cv| top_cv.container.clone()).unwrap();
                        let target_is_root = data.target_container.as_ref().map_or_else(|| false, |t| t.id_equals(&root_container));
                        let target_in_source = data.target_container.as_ref().map_or_else(|| false, |c| data.source.find(c.get_self_item()).is_some());
                        // Target is the topmost container, so we're forced to rebuild everything
                        if target_is_root {
                            // Drain all after the first
                            self.frame_handlers.container_frame_handlers.drain(1..);
                            // Then rebuild the remaining (root container) view
                            if let Some(topmost_view) = self.frame_handlers.container_frame_handlers.last_mut() {
                                topmost_view.rebuild_to_container(data.target_container.as_ref().unwrap().clone())
                            }
                        } else {
                            // i.e moving to a parent container that isn't the root
                            if !target_in_source {
                                // Rebuild that specific container view
                                if let Some(cv) = self.frame_handlers.container_frame_handlers.iter_mut()
                                    .find(|cv| cv.container.id_equals(&data.target_container.as_ref().unwrap())) {
                                    cv.rebuild_to_container(data.target_container.as_ref().unwrap().clone())
                                }
                            }

                            // Ensure the current view updates
                            if let Some(topmost_view) = self.frame_handlers.container_frame_handlers.last_mut() {
                                topmost_view.handle_callback_result(r);
                            }
                        }
                    } else {
                        for fh in &mut self.frame_handlers.container_frame_handlers {
                            if fh.container.id_equals(&data.source) {
                                fh.handle_callback_result(r.clone())
                            }
                        }
                    }
                },
                _ => {
                    self.frame_handlers.container_frame_handlers.last_mut().map(|fh| fh.handle_callback_result(r));

                }
            }
        }
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, WorldContainerViewFrameData> for WorldContainerViewFrameHandlers {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<WorldContainerViewFrameData>) {
        let main_area = data.ui_areas.get_area(UI_AREA_NAME_MAIN).unwrap().area;
        if let Some(cfh) = &mut self.choice_frame_handler {
            let inventory_area = Area::new(
                Position::new(main_area.start_position.x + 2, main_area.start_position.y + 2),
                1,
                main_area.width / 2
            );
            let frame_data = FrameData { data: Vec::new(), frame_area: inventory_area, ui_areas: data.ui_areas };
            cfh.handle_frame(frame, frame_data);
        } else if let Some(topmost_view) = self.container_frame_handlers.last_mut() {
            let mut frame_inventory = topmost_view.container.clone();
            topmost_view.handle_frame(frame, FrameData { data: &mut frame_inventory, frame_area: data.frame_area, ui_areas: data.ui_areas });
        }
    }
}

impl WorldContainerViewFrameHandlers {

    fn build_container_choice_view(&mut self, data : &MoveToContainerChoiceData) {
        if !data.choices.is_empty() {
            let choices = data.choices.clone();
            let mut items = Vec::new();
            for c in &choices {
                items.push(c.get_self_item().clone());
            }
            let cfh = ContainerChoiceFrameHandler::build(choices);

            self.choice_frame_handler = Some(cfh);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::input::IoKeyInputResolver;
    use termion::event::Key;
    use tui::backend::TestBackend;
    use crate::map::map_generator::build_dev_chest;
    use crate::terminal;
    use crate::terminal::terminal_manager::TerminalManager;
    use crate::ui::resolution::Resolution;
    use crate::ui::ui::{build_ui, UI};
    use crate::ui::ui_layout::UILayout;
    use crate::view::framehandler::container;
    use crate::view::model::usage_line::{UsageCommand, UsageLine};
    use crate::view::world_container_view::{WorldContainerView, WorldContainerViewFrameHandlers};
    use crate::view::{View, MIN_RESOLUTION};
    
    fn build_test_minimal_ui() -> UI {
        let mut ui = build_ui();
        let resolution = Resolution::new(MIN_RESOLUTION.width, MIN_RESOLUTION.height);
        let ui_layout = UILayout::new(resolution);
        ui.ui_layout = Some(ui_layout);
        ui
    }
    
    fn build_view<'a, B: tui::backend::Backend>(mut ui: &'a mut UI, mut terminal_manager: &'a mut TerminalManager<B>) -> WorldContainerView<'a, B> {
        let container = build_dev_chest();
        let subview_container = container.clone();
        let view_container = container.clone();
        
        let mut commands : HashMap<Key, UsageCommand> = HashMap::new();
        commands.insert(Key::Char('o'), UsageCommand::new('o', String::from("open") ));
        commands.insert(Key::Char('t'), UsageCommand::new('t', String::from("take")) );
        let usage_line = UsageLine::new(commands);
        let container_view = container::build_container_frame_handler(subview_container, usage_line);
        
        // AND we've created a WorldContainerView to view a dev testing Chest container
        let frame_handler = WorldContainerViewFrameHandlers { container_frame_handlers: vec![container_view], choice_frame_handler: None };
        
        let mut world_container_view = WorldContainerView {
            ui,
            terminal_manager,
            frame_handlers: frame_handler,
            container: view_container,
            callback: Box::new(|_data| {None}),
            input_resolver: Box::new(IoKeyInputResolver {})
        };
        return world_container_view
    }

    #[test]
    fn test_draw() {
        // GIVEN a UI and terminal manager representing a 80x24 (MIN_RESOLUTION) screen
        let mut ui = build_test_minimal_ui();
        let mut terminal_manager = terminal::terminal_manager::init_test(MIN_RESOLUTION).unwrap();
        let mut world_container_view = build_view(&mut ui, &mut terminal_manager);

        // WHEN we call to draw the world container view, it should complete successfully
        world_container_view.draw(None).expect("World container view should have been drawn");
    }
}