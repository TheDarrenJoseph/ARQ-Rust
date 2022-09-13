use std::collections::HashSet;
use std::convert::TryInto;
use std::io;
use std::io::Error;
use std::slice::Iter;

use termion::event::Key;
use termion::input::TermRead;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::symbols::line::VERTICAL;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs};

use crate::character::{Character};
use crate::list_selection::ListSelection;
use crate::map::position::Area;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::{FrameData, FrameHandler, UI};
use crate::view::{GenericInputResult, resolve_input, View};
use crate::view::callback::Callback;
use crate::view::framehandler::character::{CharacterFrameHandler, ViewMode};
use crate::view::framehandler::container;
use crate::view::framehandler::container::{ContainerFrameHandler, ContainerFrameHandlerCommand, ContainerFrameHandlerInputResult, MoveItemsData, MoveToContainerChoiceData};
use crate::view::framehandler::container::ContainerFrameHandlerCommand::{DROP, OPEN};
use crate::view::framehandler::container_choice::{build, ContainerChoiceFrameHandler, ContainerChoiceFrameHandlerInputResult};
use crate::view::InputHandler;

#[derive(PartialEq, Clone, Debug)]
pub enum TabChoice {
    INVENTORY,
    CHARACTER
}

impl TabChoice {
    pub fn iterator() -> Iter<'static, TabChoice> {
        [TabChoice::INVENTORY, TabChoice::CHARACTER].iter()
    }
}

struct CharacterInfoViewFrameData {
    pub character : Character
}

// Combines multiple character info views into one w/ tabbing
pub struct CharacterInfoView<'a, B : tui::backend::Backend> {
    pub character : &'a mut Character,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: CharacterInfoViewFrameHandler,
    pub callback : Box<dyn FnMut(ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> + 'a>
}

pub struct CharacterInfoViewFrameHandler {
    pub tab_choice : TabChoice,
    pub container_frame_handlers: Vec<ContainerFrameHandler>,
    pub choice_frame_handler: Option<ContainerChoiceFrameHandler>,
    pub character_view : Option<CharacterFrameHandler>
}

impl <B : tui::backend::Backend> CharacterInfoView<'_, B> {
    fn initialise(&mut self) {
        let mut commands : HashSet<ContainerFrameHandlerCommand> = HashSet::new();
        commands.insert(OPEN);
        commands.insert(DROP);
        let inventory_view = container::build_container_frame_handler(self.character.get_inventory_mut().clone(), commands);
        self.frame_handler.container_frame_handlers = vec!(inventory_view);

        let character_view = CharacterFrameHandler { character: self.character.clone(), widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::VIEW };
        self.frame_handler.character_view = Some(character_view);
    }

    // TODO refactor alongside other commands / engine func
    fn re_render(&mut self) -> Result<(), std::io::Error>  {
        let ui = &mut self.ui;
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
        })?;
        Ok(())
    }

    fn next_tab(&mut self)  {
        let tab_iter = TabChoice::iterator();
        if let Some(max_index) = tab_iter.size_hint().1 {
            let mut index = 0;
            let mut target_index = None;
            for tab_choice in tab_iter {
                let current_choice = self.frame_handler.tab_choice.clone();
                if *tab_choice == current_choice && index == max_index - 1 {
                    // Swap back to the first option
                    if let Some(choice) = TabChoice::iterator().next() {
                        self.frame_handler.tab_choice = choice.clone();
                    }
                } else if *tab_choice == current_choice {
                    target_index = Some(index.clone() + 1);
                }
                index += 1;
            }

            // Select the target tab choice otherwise
            if let Some(idx) = target_index {
                if let Some(tab_choice) = TabChoice::iterator().nth(idx) {
                    self.frame_handler.tab_choice = tab_choice.clone();
                }
            }
        }
    }

    fn handle_callback_result(&mut self, result: Option<ContainerFrameHandlerInputResult>) {
        if let Some(r) = result {
            match r {
                ContainerFrameHandlerInputResult::MoveToContainerChoice(ref data) => {
                    if !data.choices.is_empty() {
                        let choices = data.choices.clone();
                        let mut items = Vec::new();
                        for c in &choices {
                            items.push(c.get_self_item().clone());
                        }
                        let mut cfh = build(choices);
                        self.frame_handler.choice_frame_handler = Some(cfh);
                    }
                },
                ContainerFrameHandlerInputResult::MoveItems(ref data) => {

                    if data.target_container.is_some() {
                        // if target_container is the root view container
                        let root_container = self.frame_handler.container_frame_handlers.first().map(|top_cv| top_cv.container.clone()).unwrap();
                        let target_is_root = data.target_container.as_ref().map_or_else(|| false, |t| t.id_equals(&root_container));
                        let target_in_source = data.target_container.as_ref().map_or_else(|| false, |c| data.source.find(c.get_self_item()).is_some());
                        // Target is the topmost container, so we're forced to rebuild everything
                        if target_is_root {
                            // Drain all after the first
                            self.frame_handler.container_frame_handlers.drain(1..);
                            // Then rebuild the remaining (root container) view
                            if let Some(topmost_view) = self.frame_handler.container_frame_handlers.last_mut() {
                                topmost_view.rebuild_to_container(data.target_container.as_ref().unwrap().clone())
                            }
                        } else {
                            // i.e moving to a parent container that isn't the root
                            if !target_in_source {
                                // Rebuild that specific container view
                                if let Some(cv) = self.frame_handler.container_frame_handlers.iter_mut()
                                    .find(|cv| cv.container.id_equals(&data.target_container.as_ref().unwrap())) {
                                    cv.rebuild_to_container(data.target_container.as_ref().unwrap().clone())
                                }
                            }

                            // Ensure the current view updates
                            if let Some(topmost_view) = self.frame_handler.container_frame_handlers.last_mut() {
                                topmost_view.handle_callback_result(r);
                            }
                        }
                    } else {
                        for fh in &mut self.frame_handler.container_frame_handlers {
                            if fh.container.id_equals(&data.source) {
                                fh.handle_callback_result(r.clone())
                            }
                        }
                    }
                }
                _ => {
                    // Find source view and update it
                    if let Some(topmost_view) = self.frame_handler.container_frame_handlers.last_mut() {
                        topmost_view.handle_callback_result(r);
                    }
                }
            }
        }
    }

    /*
      * Handles passthrough to the relevant container choice view (if one is present)
      * Triggering callbacks if needed
      * Returns the optional input result of the container choice view, and a boolean to indicate success
      */
    fn handle_container_choice_input(&mut self, key: Key) -> Result<(Option<GenericInputResult>, bool), Error> {
        if let Some(cfh) = &mut self.frame_handler.choice_frame_handler {
            let result = cfh.handle_input(Some(key))?;
            if let Some(view_specific_result) = result.view_specific_result {
                match view_specific_result {
                    ContainerChoiceFrameHandlerInputResult::Select(selected_container) => {
                        let container_views = &mut self.frame_handler.container_frame_handlers;
                        if let Some(topmost_view) = container_views.last_mut() {
                            let mut view_specific_result = topmost_view.build_move_items_result().unwrap().view_specific_result.unwrap();
                            match view_specific_result {
                                ContainerFrameHandlerInputResult::MoveToContainerChoice(mut data) => {
                                    // Add the target selected to the callback data
                                    data.target_container = Some(selected_container.clone());
                                    self.trigger_callback(ContainerFrameHandlerInputResult::MoveToContainerChoice(data));
                                    // Clear the frame handler now we're done
                                    self.frame_handler.choice_frame_handler = None;
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

    /*
     * Handles passthrough to the relevant container view (the latest opened)
     * Triggering callbacks if needed
     * Returns the optional input result of the container view, and a boolean to indicate success
     */
    fn handle_container_view_input(&mut self, key: Key) -> Result<(Option<GenericInputResult>, bool), Error> {
        let container_views = &mut self.frame_handler.container_frame_handlers;
        let have_container_views = !container_views.is_empty();
        if have_container_views {
            if let Some(topmost_view) = container_views.last_mut() {
                let result = topmost_view.handle_input(Some(key))?;
                if let Some(view_specific_result) = result.view_specific_result {
                    match view_specific_result {
                        ContainerFrameHandlerInputResult::OpenContainerView(stacked_view) => {
                            container_views.push(stacked_view);
                        },
                        ContainerFrameHandlerInputResult::DropItems(_) => {
                            self.trigger_callback(view_specific_result);
                        },
                        ContainerFrameHandlerInputResult::MoveItems(_) => {
                            self.trigger_callback(view_specific_result);
                        },
                        ContainerFrameHandlerInputResult::MoveToContainerChoice(_) => {
                            self.trigger_callback(view_specific_result);
                        },
                        _ => {}
                    }
                }
                return Ok((Some(result.generic_input_result), true));
            }
        }
        Ok((None, false))
    }

    fn quit_container_view(&mut self) -> Result<bool, Error> {
        let container_views = &mut self.frame_handler.container_frame_handlers;
        if container_views.len() > 1 {
            if let Some(closing_view) = self.frame_handler.container_frame_handlers.pop() {
                let closing_container = closing_view.container;
                if let Some(parent_view) = self.frame_handler.container_frame_handlers.last_mut() {
                    let parent_container = &mut parent_view.container;
                    if let Some(position) = parent_container.position(&closing_container) {
                        parent_container.replace(position, closing_container);
                    }
                }
            }
            return Ok(false)
        } else if container_views.len() == 1 {
            let last_view = &mut self.frame_handler.container_frame_handlers[0];
            self.character.set_inventory(last_view.container.clone());
        }
        return Ok(true)
    }
}

impl <'c, B : tui::backend::Backend> Callback<'c, ContainerFrameHandlerInputResult> for CharacterInfoView<'c, B> {
    fn set_callback(&mut self, callback: Box<impl FnMut(ContainerFrameHandlerInputResult) -> Option<ContainerFrameHandlerInputResult> + 'c>) {
        self.callback = callback;
    }

    fn trigger_callback(&mut self, data: ContainerFrameHandlerInputResult) {
        let result = (self.callback)(data);
        self.handle_callback_result(result);
    }
}

impl <'b, B : tui::backend::Backend> View<'b, GenericInputResult> for CharacterInfoView<'_, B>  {
    fn begin(&mut self)  -> Result<bool, Error> {
        self.initialise();
        self.terminal_manager.terminal.clear()?;
        self.draw(None)?;
        while !self.handle_input(None).unwrap() {
            self.draw(None)?;
        }
        Ok(true)
    }

    fn draw(&mut self, _area: Option<Area>) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        let character = self.character.clone();
        let ui = &mut self.ui;

        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
            let areas = ui.get_view_areas(frame.size());
            let view_area = areas[0];
            // Sizes for the entire 'Character Info' frame area
            let frame_area = Rect { x : view_area.x, y : view_area.y +1 , width: view_area.width.clone(),  height: view_area.height.clone() - 1};
            let specific_frame_data = CharacterInfoViewFrameData { character };
            frame_handler.handle_frame(frame, FrameData { frame_size: frame_area, data: specific_frame_data });
        })?;
        Ok(())
    }

    fn handle_input(&mut self, input: Option<Key>) -> Result<bool, Error> {
        let key = resolve_input(input);
        match key {
            Key::Char('q') => {
                return self.quit_container_view();
            },
            // Horizontal tab
            Key::Char('\t') => {
                self.next_tab();
            }
            // Passthrough anything not handled here into the sub views
            _ => {
                let mut generic_input_result : Option<GenericInputResult> = None;
                match self.frame_handler.tab_choice {
                    TabChoice::INVENTORY => {
                        // Container choice handlers take priority as they're essentially a dialog
                        if let Ok((gir_result, success)) = self.handle_container_choice_input(key) {
                            // Rewrap this only if something was returned
                            if let Some(gir) = gir_result {
                                generic_input_result = Some(gir);
                            }
                            // Force a redraw
                            if success {
                                return Ok(false);
                            }
                        }

                        // Finally trigger passthrough to standard container views
                        if let Ok((gir_result, success)) = self.handle_container_view_input(key) {
                            // Rewrap this only if something was returned
                            if let Some(gir) = gir_result {
                                generic_input_result = Some(gir);
                            }
                        }
                    }
                    TabChoice::CHARACTER => {
                        // TODO future pass-through to character details view??
                    }
                }

                if let Some(r) = generic_input_result {
                    if r.requires_view_refresh {
                        self.terminal_manager.terminal.clear()?;
                    }
                }

            }
        }

        return Ok(false)
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, CharacterInfoViewFrameData> for CharacterInfoViewFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<CharacterInfoViewFrameData>) {
        let titles =  ["Inventory", "Character"].iter().cloned().map(Spans::from).collect();
        let selection_index = self.tab_choice.clone() as i32;
        let tabs = Tabs::new(titles)
            .block(Block::default().title("Character Info").borders(Borders::NONE))
            .style(Style::default())
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .divider(VERTICAL)
            .select(selection_index as usize);

        let frame_size = data.frame_size;
        let heading_area = Rect::new(frame_size.x + 1, frame_size.y, frame_size.width - 2, 3);
        frame.render_widget(tabs, heading_area);

        let character = data.data.character;
        match self.tab_choice {
            TabChoice::INVENTORY => {
                if let Some(cfh) = &mut self.choice_frame_handler {
                    let inventory_area = Rect::new(frame_size.x + 1, frame_size.y + 2, frame_size.width - 2, frame_size.height - 3);
                    let frame_data = FrameData { data: Vec::new(), frame_size: inventory_area};
                    cfh.handle_frame(frame, frame_data);
                } else if let Some(topmost_view) = self.container_frame_handlers.last_mut() {
                    let mut frame_inventory = topmost_view.container.clone();
                    let inventory_area = Rect::new(frame_size.x + 1, frame_size.y + 2, frame_size.width - 2, frame_size.height - 3);
                    topmost_view.handle_frame(frame, FrameData { frame_size: inventory_area, data: &mut frame_inventory });
                }
            },
            TabChoice::CHARACTER => {
                match &mut self.character_view {
                    Some(char_view) => {
                        let character_area = Rect::new(frame_size.x + 1, frame_size.y + 2, frame_size.width - 2, frame_size.height - 3);
                        char_view.handle_frame(frame,  FrameData { frame_size: character_area, data: character.clone() } );
                    },
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::character::{build_character, build_default_character_details, Character};
    use crate::engine::level::{Characters, Level};
    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::objects::items;
    use crate::map::position::{build_square_area, Position};
    use crate::map::tile::{Colour, Tile};
    use crate::terminal::terminal_manager;
    use crate::ui::build_ui;
    use crate::view::character_info::{CharacterInfoView, CharacterInfoViewFrameHandler, TabChoice};
    use crate::view::framehandler::container::ContainerFrameHandlerInputResult;

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container = build(id, "Test Container".to_owned(), 'X', 1, 1, ContainerType::OBJECT, 100);
        let container_self_item = container.get_self_item();
        assert_eq!(id, container_self_item.get_id());
        assert_eq!("Test Container", container_self_item.name);
        assert_eq!('X', container_self_item.symbol);
        assert_eq!(Colour::White, container_self_item.colour);
        assert_eq!(1, container_self_item.weight);
        assert_eq!(1, container_self_item.value);

        for i in 1..=4 {
            let test_item = items::build_item(Uuid::new_v4(), format!("Test Item {}", i), 'X', 1, 100);
            container.add_item(test_item);
        }

        assert_eq!(ContainerType::OBJECT, container.container_type);
        assert_eq!(100, container.get_weight_limit());
        let contents = container.get_contents();
        assert_eq!(4, contents.len());
        container
    }

    fn build_test_level(player: Character) -> Level {
        let tile_library = crate::map::tile::build_library();
        let rom = tile_library[&Tile::Room].clone();
        let wall = tile_library[&Tile::Wall].clone();
        let map_pos = Position { x: 0, y: 0 };
        let map_area = build_square_area(map_pos, 3);

        let map = crate::map::Map {
            area: map_area,
            tiles: vec![
                vec![wall.clone(), wall.clone(), wall.clone()],
                vec![wall.clone(), rom.clone(), wall.clone()],
                vec![wall.clone(), wall.clone(), wall.clone()],
            ],
            rooms: Vec::new(),
            containers: HashMap::new()
        };

        return Level { map: Some(map), characters: Characters { characters: vec![player] } };
    }

    #[test]
    fn test_initialise() {
        // GIVEN a valid character info view for a player's inventory
        let inventory = build(Uuid::new_v4(), "Test Player's Inventory".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 2);
        let character_details = build_default_character_details();
        let mut player = build_character(String::from("Test Player") , Position { x: 0, y: 0}, inventory);
        let mut level = build_test_level(player);

        let mut ui = build_ui();
        let mut terminal_manager = terminal_manager::init_test().unwrap();
        let frame_handler = CharacterInfoViewFrameHandler { tab_choice: TabChoice::INVENTORY, container_frame_handlers: Vec::new(), choice_frame_handler: None, character_view: None };
        let mut character_info_view = CharacterInfoView { character: level.characters.get_player_mut(), ui: &mut ui, terminal_manager: &mut terminal_manager, frame_handler, callback: Box::new(|data| {None}) };

        // WHEN we call to initialise
        // THEN we expect it to complete successfully
        character_info_view.initialise();
    }
}