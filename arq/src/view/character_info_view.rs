use std::io;
use std::io::Error;
use tui::layout::{Rect};
use tui::text::{Spans, Span};
use tui::style::{Style, Color, Modifier};
use tui::symbols::line::VERTICAL;
use tui::buffer::{Buffer};
use tui::widgets::{Tabs, Block, Borders};
use termion::input::TermRead;
use termion::event::Key;
use std::slice::Iter;

use crate::ui::{UI, FrameHandler, FrameData};
use crate::view::{View, resolve_input};
use crate::terminal::terminal_manager::TerminalManager;
use crate::character::{get_all_attributes, Character, Race, Class, determine_class, Attribute};
use crate::widget::text_widget::build_text_input;
use crate::widget::dropdown_widget::{build_dropdown, DropdownInputState};
use crate::widget::number_widget::{build_number_input, build_number_input_with_value, NumberInputState};
use crate::widget::button_widget::build_button;
use crate::widget::character_stat_line::{build_character_stat_line, CharacterStatLineState};
use crate::widget::{Focusable, Widget, WidgetType, Named};
use crate::character;
use crate::view::character_view::{CharacterView, CharacterViewFrameHandler, ViewMode};
use crate::view::container_view::{ContainerView, ContainerFrameHandler, build_container_view};
use crate::map::position::Area;

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

// Combines multiple character info views into one w/ tabbing
pub struct CharacterInfoView<'a, B : tui::backend::Backend> {
    pub character : &'a mut Character,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub frame_handler: CharacterInfoViewFrameHandler<'a, B>
}

pub struct CharacterInfoViewFrameHandler<'a, B : tui::backend::Backend> {
    pub tab_choice : TabChoice,
    pub character_view : Option<CharacterView<'a, B>>
}

impl <B : tui::backend::Backend> CharacterInfoView<'_, B> {


    pub(crate) fn begin(&mut self) {
        self.draw(None);
        while !self.handle_input(None).unwrap() {
            self.draw(None);
        }
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
}

impl <B : tui::backend::Backend> View for CharacterInfoView<'_, B>  {
    fn draw(&mut self, area: Option<Rect>) -> Result<(), Error> {
        self.terminal_manager.terminal.clear();
        let frame_handler = &mut self.frame_handler;
        let character = self.character.clone();
        let ui = &mut self.ui;
        let terminal_manager = &mut self.terminal_manager;
        let tab_choice = frame_handler.tab_choice.clone();

        let mut inventory_character = self.character.clone();


        let mut frame_area = Rect::default();
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(frame);
            let size = frame.size();
            frame_area = Rect { x : size.x.clone() + 1, y : size.y.clone() + 2, width: size.width.clone() -2,  height: size.height.clone() - 2};
            frame_handler.handle_frame(frame, FrameData { frame_size: frame.size(), data: character });

            match tab_choice {
                TabChoice::INVENTORY => {
                    let mut inventory = inventory_character.get_inventory().clone();
                    // TODO use a pure frame handler instead of a view?
                    //let mut inventory_view = build_container_view(&mut inventory, ui, terminal_manager);
                }
                _ => {}
            }
        })?;

        /** TODO
        match tab_choice {
            TabChoice::INVENTORY => {
                let mut inventory = self.character.get_inventory().clone();
                let mut inventory_view = build_container_view( &mut inventory, &mut self.ui, &mut self.terminal_manager);
                //inventory_view.draw(Some(frame_area));
                // Update to reflect any changes
                self.character.set_inventory(inventory.clone());
            }
            TabChoice::CHARACTER => {
                let frame_handler = CharacterViewFrameHandler { widgets: Vec::new(), selected_widget: None, view_mode: ViewMode::VIEW };
                let mut character_view = CharacterView { character: self.character.clone(), ui: self.ui, terminal_manager: self.terminal_manager, frame_handler};
                //character_view.draw(None);
            }
        }**/

        Ok(())
    }

    fn handle_input(&mut self, input: Option<Key>) -> Result<bool, Error> {
        let key = resolve_input(input);
        let _horizontal_tab = char::from_u32(0x2409);
        match key {
            Key::Char('q') => {
                return Ok(true)
            },
            // Horizontal tab
            Key::Char(_horizontal_tab) => {
                self.next_tab();
            }
            _ => {
                match self.frame_handler.tab_choice {
                    TabChoice::INVENTORY => {
                        // TODO passthrough to inventory view
                    }
                    TabChoice::CHARACTER => {
                        // TODOD passthorugh to character details view
                    }
                }
            }
        }
        return Ok(false)
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, Character> for CharacterInfoViewFrameHandler<'_, B> {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<Character>) {
        let titles =  ["Inventory", "Character"].iter().cloned().map(Spans::from).collect();
        let selection_index = self.tab_choice.clone() as i32;
        let mut tabs = Tabs::new(titles)
            .block(Block::default().title("Character Info").borders(Borders::ALL))
            .style(Style::default())
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .divider(VERTICAL)
            .select(selection_index as usize);

        let frame_size = frame.size();
        let tab_area = Rect::new(1, 1, frame_size.width - 4, frame_size.height - 4);
        frame.render_widget(tabs, tab_area);
    }
}