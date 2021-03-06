use termion::event::Key;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::symbols::line::VERTICAL;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs};

use std::io::Error;
use std::slice::Iter;
use std::collections::HashSet;

use crate::character::{Attribute, Character, Class, determine_class, get_all_attributes, Race};
use crate::character;
use crate::map::position::Area;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::{FrameData, FrameHandler, UI};
use crate::view::{GenericInputResult, InputResult, resolve_input, View};
use crate::view::framehandler::character::{CharacterFrameHandler, ViewMode};
use crate::view::framehandler::container;
use crate::view::framehandler::container::{build_container_view, ContainerFrameHandler, ContainerFrameHandlerInputResult, ContainerFrameHandlerCommand};
use crate::view::InputHandler;
use crate::widget::{Focusable, Named, Widget, WidgetType};
use crate::widget::button_widget::build_button;
use crate::widget::character_stat_line::{build_character_stat_line, CharacterStatLineState};
use crate::widget::dropdown_widget::{build_dropdown, DropdownInputState};
use crate::widget::number_widget::{build_number_input, build_number_input_with_value, NumberInputState};
use crate::widget::text_widget::build_text_input;
use crate::view::framehandler::container::ContainerFrameHandlerCommand::{OPEN, DROP};
use crate::view::callback::Callback;
use crate::view::framehandler::container::ContainerFrameHandlerInputResult::DropItems;

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
    pub container_views : Vec<ContainerFrameHandler>,
    pub character_view : Option<CharacterFrameHandler>
}

impl <B : tui::backend::Backend> CharacterInfoView<'_, B> {
    fn initialise(&mut self) {
        let mut commands : HashSet<ContainerFrameHandlerCommand> = HashSet::new();
        commands.insert(OPEN);
        commands.insert(DROP);
        let inventory_view = container::build_container_view(self.character.get_inventory().clone(), commands);
        self.frame_handler.container_views = vec!(inventory_view);

        let mut character_view = CharacterFrameHandler { character: self.character.clone(), widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::VIEW };
        self.frame_handler.character_view = Some(character_view);
    }

    // TODO refactor alongside other commands / engine func
    fn re_render(&mut self) -> Result<(), std::io::Error>  {
        let mut ui = &mut self.ui;
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
            if let Some(topmost_view) = self.frame_handler.container_views.last_mut() {
                topmost_view.handle_callback_result(r);
            }
        }
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


struct CharacterViewInputResult {

}

impl <'b, B : tui::backend::Backend> View<'b, GenericInputResult> for CharacterInfoView<'_, B>  {
    fn begin(&mut self)  -> Result<bool, Error> {
        self.initialise();
        self.terminal_manager.terminal.clear();
        self.draw(None);
        while !self.handle_input(None).unwrap() {
            self.draw(None);
        }
        Ok(true)
    }


    fn draw(&mut self, area: Option<Area>) -> Result<(), Error> {
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
                    self.character.set_inventory(last_view.container.clone());
                }
                return Ok(true)
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
                        let mut container_views = &mut self.frame_handler.container_views;
                        let have_container_views = !container_views.is_empty();
                        if have_container_views {
                            if let Some(topmost_view) = container_views.last_mut() {
                                let mut container_view_input_result = topmost_view.handle_input(Some(key));
                                let result = container_view_input_result.unwrap();
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
                                        _ => {}
                                    }
                                }
                                generic_input_result = Some(result.generic_input_result);
                            }
                        }
                    }
                    TabChoice::CHARACTER => {
                        // TODO future pass-through to character details view??
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

impl <B : tui::backend::Backend> FrameHandler<B, CharacterInfoViewFrameData> for CharacterInfoViewFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<CharacterInfoViewFrameData>) {
        let titles =  ["Inventory", "Character"].iter().cloned().map(Spans::from).collect();
        let selection_index = self.tab_choice.clone() as i32;
        let mut tabs = Tabs::new(titles)
            .block(Block::default().title("Character Info").borders(Borders::NONE))
            .style(Style::default())
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .divider(VERTICAL)
            .select(selection_index as usize);

        let frame_size = data.frame_size;
        let heading_area = Rect::new(frame_size.x + 1, frame_size.y, frame_size.width - 2, 3);
        frame.render_widget(tabs, heading_area);

        let mut character = data.data.character;
        match self.tab_choice {
            TabChoice::INVENTORY => {
                if let Some(topmost_view) = self.container_views.last_mut() {
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
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use std::collections::HashMap;

    use crate::view::character_info::{CharacterInfoView, CharacterInfoViewFrameHandler, TabChoice};
    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::tile::{Colour, Tile};
    use crate::map::objects::items;
    use crate::character::{Character, build_default_character_details, build_character};
    use crate::engine::level::Level;
    use crate::map::position::{build_square_area, Position};
    use crate::terminal::terminal_manager;
    use crate::ui::build_ui;
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

        return Level { map: Some(map), characters: vec![player] };
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
        let frame_handler = CharacterInfoViewFrameHandler { tab_choice: TabChoice::INVENTORY, container_views: Vec::new(), character_view: None };
        let mut character_info_view = CharacterInfoView { character: level.get_player_mut(), ui: &mut ui, terminal_manager: &mut terminal_manager, frame_handler, callback: Box::new(|data| {None}) };

        // WHEN we call to initialise
        // THEN we expect it to complete successfully
        character_info_view.initialise();
    }
}