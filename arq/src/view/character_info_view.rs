use termion::event::Key;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::symbols::line::VERTICAL;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs};

use std::io::Error;
use std::slice::Iter;

use crate::character::{Attribute, Character, Class, determine_class, get_all_attributes, Race};
use crate::character;
use crate::map::position::Area;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::{FrameData, FrameHandler, UI};
use crate::view::{GenericInputResult, InputResult, resolve_input, View};
use crate::view::framehandler::character_view::{CharacterView, ViewMode};
use crate::view::framehandler::container_view;
use crate::view::framehandler::container_view::{build_container_view, ContainerView, ContainerViewInputResult};
use crate::view::InputHandler;
use crate::widget::{Focusable, Named, Widget, WidgetType};
use crate::widget::button_widget::build_button;
use crate::widget::character_stat_line::{build_character_stat_line, CharacterStatLineState};
use crate::widget::dropdown_widget::{build_dropdown, DropdownInputState};
use crate::widget::number_widget::{build_number_input, build_number_input_with_value, NumberInputState};
use crate::widget::text_widget::build_text_input;

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
    pub frame_handler: CharacterInfoViewFrameHandler
}

pub struct CharacterInfoViewFrameHandler {
    pub tab_choice : TabChoice,
    pub container_views : Vec<ContainerView>,
    pub character_view : Option<CharacterView>
}

impl <B : tui::backend::Backend> CharacterInfoView<'_, B> {

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
}

struct CharacterViewInputResult {

}

impl <'b, B : tui::backend::Backend> View<'b, GenericInputResult> for CharacterInfoView<'_, B>  {
    fn begin(&mut self)  -> Result<bool, Error> {
        let inventory_view = container_view::build_container_view( self.character.get_inventory().clone());
        self.frame_handler.container_views = vec!(inventory_view);

        let mut character_view = CharacterView { character: self.character.clone(), widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::VIEW };
        self.frame_handler.character_view = Some(character_view);

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

        let mut frame_area = Rect::default();
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
            let size = frame.size();
            frame_area = Rect { x : size.x.clone() + 1, y : size.y.clone() + 2, width: size.width.clone() -2,  height: size.height.clone() - 2};

            let specific_frame_data = CharacterInfoViewFrameData { character };
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
                                if let Some(ContainerViewInputResult::OPEN_CONTAINER_VIEW(stacked_view)) = result.view_specific_result {
                                    container_views.push(stacked_view);
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
            .block(Block::default().title("Character Info").borders(Borders::ALL))
            .style(Style::default())
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .divider(VERTICAL)
            .select(selection_index as usize);

        let frame_size = frame.size();
        let tab_area = Rect::new(frame_size.x + 1, frame_size.y + 1, frame_size.width - 2, frame_size.height - 2);
        frame.render_widget(tabs, tab_area);


        let mut character = data.data.character;
        match self.tab_choice {
            TabChoice::INVENTORY => {
                if let Some(topmost_view) = self.container_views.last_mut() {
                    let mut frame_inventory = topmost_view.container.clone();
                    let inventory_area = Rect::new(2, 3, frame_size.width - 4, frame_size.height - 5);
                    topmost_view.handle_frame(frame, FrameData { frame_size: inventory_area, data: &mut frame_inventory });
                }
            },
            TabChoice::CHARACTER => {
                match &mut self.character_view {
                    Some(char_view) => {
                        char_view.handle_frame(frame,  FrameData { frame_size: frame.size(), data: character.clone() } );
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}