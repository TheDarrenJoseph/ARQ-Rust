use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fmt::format;
use std::io::Error;
use std::ptr::eq;

use termion::event::Key;
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};

use tui::widgets::{Block, Borders};

use crate::list_selection::{build_list_selection, ItemListSelection, ListSelection};
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::map::position::Position;
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::view::framehandler::container_choice::ContainerChoiceCommand;

use crate::view::framehandler::util::paging::{build_page_count, build_weight_limit};
use crate::view::framehandler::util::tabling::{build_headings, build_paragraph, Column};
use crate::view::model::usage_line::{UsageCommand, UsageLine};

/*
    This frame handler is meant to display containers (Chests, Floor items, Dead bodies) in a tabular display
    This allows opening nested containers, and provides callbacks to perform operations on objects within them (Take, Drop, Use, Equip, etc)
 */
#[derive(Clone)]
pub struct ContainerFrameHandler {
    pub container : Container,
    columns : Vec<Column>,
    row_count: i32,
    pub item_list_selection : ItemListSelection,
    usage_line : UsageLine
}

#[derive(Clone)]
pub struct TakeItemsData {
    pub source: Container,
    pub to_take: Vec<Item>,
    pub position: Option<Position>
}

#[derive(Clone)]
pub struct MoveToContainerChoiceData {
    pub source: Container,
    pub to_move: Vec<Item>,
    pub position: Option<Position>,
    pub choices: Vec<Container>,
    pub target_container: Option<Container>
}

#[derive(Clone)]
pub struct MoveItemsData {
    pub source: Container,
    pub to_move: Vec<Item>,
    pub target_container: Option<Container>,
    pub target_item: Option<Item>,
    pub position: Option<Position>
}

#[derive(Clone)]
pub enum ContainerFrameHandlerInputResult {
    None,
    OpenContainerView(ContainerFrameHandler),
    MoveToContainerChoice(MoveToContainerChoiceData),
    MoveItems(MoveItemsData),
    TakeItems(TakeItemsData),
    DropItems(Vec<Item>),
    EquipItems(Vec<Item>)
}

fn build_default_columns() -> Vec<Column> {
    vec![
        Column {name : "NAME".to_string(), size: 30},
        Column {name : "WEIGHT (Kg)".to_string(), size: 12},
        Column {name : "VALUE".to_string(), size: 12}
    ]
}

fn build_column_text(column: &Column, item: &Item) -> String {
    match column.name.as_str() {
        "NAME" => {
            if item.is_equipped() {
                format!("{} ({})", item.get_name(), item.get_equipment_slot().unwrap())
            } else {
                item.get_name()
            }
        },
        "WEIGHT (Kg)" => {
            item.get_weight().to_string()
        },
        "VALUE" => {
            item.get_value().to_string()
        },
        _ => { "".to_string() }
    }
}


pub fn build_container_frame_handler(container: Container, usage_line : UsageLine) -> ContainerFrameHandler {
    let columns = build_default_columns();
    let items = container.to_cloned_item_list();
    ContainerFrameHandler {
        container: container.clone(),
        columns,
        row_count: 1,
        item_list_selection: build_list_selection(items, 1),
        usage_line
    }
}

impl ContainerFrameHandler {

    fn find_focused_container(&mut self) -> Option<Container> {
        let list_selection = &self.item_list_selection;
        if list_selection.is_selecting() {
            let focused_item_result = list_selection.get_focused_item();
            if let Some(focused_item) = focused_item_result {
                // Make sure we've not focused any of the selected items
                let selected_container_items = self.get_selected_containers();
                let focused_items = selected_container_items.iter().find(|ci| ci.get_self_item().get_id() == focused_item.get_id());
                if let None = focused_items {
                    if focused_item.is_container() {
                        if let Some(c) = self.container.find_mut(focused_item) {
                            return Some(c.clone());
                        }
                    }
                }
            }
        }
        None
    }


    fn find_focused_item(&mut self) -> Option<Item> {
        let list_selection = &self.item_list_selection;
        if let Some(focused_item) = list_selection.get_focused_item() {
            return Some(focused_item.clone());
        }
        None
    }

    fn replace_focused_container(&mut self, updated: Container) {
        if let Some(mut focused) = self.find_focused_container()
        {
            focused.replace_container(updated);
        }
    }

    pub fn cancel_selection(&mut self) {
        self.item_list_selection.cancel_selection();
    }

    pub fn rebuild_selection(&mut self) {
        self.item_list_selection = build_list_selection(self.container.to_cloned_item_list(), 1);
    }

    pub fn rebuild_selection_from(&mut self, container: &Container) {
        self.item_list_selection = build_list_selection(container.to_cloned_item_list(), 1);
    }

    pub fn get_selected_items(&self) -> Vec<Item> {
        Vec::from(self.item_list_selection.get_selected_items().clone())
    }

    fn find_container_for_item(&self, item: &Item) -> Option<&Container> {
        self.container.find(item)
    }

    fn find_selected_containers(&self, selected_items : Vec<Item>) -> Vec<Container> {
        let mut found = Vec::new();
        for selected_item in selected_items.iter() {
            // Collect our 'Container' wrappers matching the selection
            if let Some(found_container) = self.find_container_for_item(&selected_item) {
                found.push(found_container.clone());
            }
        }
        found
    }

    fn get_selected_containers(&self) -> Vec<Container> {
        let selected_items = self.get_selected_items();
        let selected_containers = self.find_selected_containers(selected_items);
        selected_containers
    }

    fn handle_quit(&mut self) -> Result<bool, Error> {
        if !self.item_list_selection.get_selected_items().is_empty() {
            self.item_list_selection.cancel_selection();
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn toggle_select(&mut self) {
        self.item_list_selection.toggle_select();
    }

    fn move_selected(&mut self) -> Result<InputResult<ContainerFrameHandlerInputResult>, Error> {
        let from_container = self.container.clone();
        let selected_container_items = self.get_selected_items();
        let focused_container = self.find_focused_container();

        let focused_item = if self.item_list_selection.is_selecting() {
            self.find_focused_item()
        } else {
            None
        };

        let container_name = if let Some(c) = focused_container.clone() { c.get_self_item().get_name() } else { String::from("N/a") };
        log::info!("Triggering MoveItems of {} items into: {}", selected_container_items.len(), container_name);

        let data = MoveItemsData { source: from_container.clone(), to_move: selected_container_items, target_container: focused_container, target_item: focused_item, position: None };
        return Ok(InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
            view_specific_result: Some(ContainerFrameHandlerInputResult::MoveItems(data))
        });
    }

    fn move_up(&mut self) {
        self.item_list_selection.move_up();
    }

    fn move_down(&mut self) {
        self.item_list_selection.move_down();
    }

    fn page_up(&mut self) {
        self.item_list_selection.page_up();
    }

    fn page_down(&mut self) {
        self.item_list_selection.page_down();
    }

    fn build_view_for_focused_container(&mut self) -> Option<ContainerFrameHandler> {
        if !self.item_list_selection.is_selecting() {
            if let Some(focused_item) = self.item_list_selection.get_focused_item() {
                if focused_item.is_container() {
                    if let Some(focused_container) = self.container.find_mut(focused_item) {
                        return Some(build_container_frame_handler(focused_container.clone(), self.usage_line.clone()))
                    }
                }
            }
        }
        None
    }

    fn clone_selected_container_items(&mut self) -> Vec<Container> {
        let mut items = Vec::new();
        let selected_items = self.get_selected_items();
        for item in selected_items {
            if let Some(found) = self.container.find(&item) {
                items.push(found.clone());
            }
        }
        items
    }

    fn retain_selected_items(&mut self, to_retain: Vec<Item>) {
        let mut droppable_containers = self.clone_selected_container_items();
        if !droppable_containers.is_empty() {
            let view_container = &mut self.container;
            for retainable in to_retain {
                if let Some(pos) = droppable_containers.iter().position(|c| *c.get_self_item() == retainable) {
                    droppable_containers.remove(pos);
                }
            }
            view_container.remove_matching_items(droppable_containers);
            self.rebuild_selection();
        }
    }

    fn equip_items(&mut self, modified: Vec<Item>) {
        log::info!("Equip modified {} items..", modified.len());
        let contents = self.container.get_contents_mut();
        for modified_item in modified {
            // Update the equipment slot of any items we're holding
            if let Some(pos) = contents.iter().position(|c| c.get_self_item().id_equals(&modified_item)) {
                let mut item = contents.get_mut(pos).unwrap().get_self_item_mut();
                item.set_equipment_slot(modified_item.get_equipment_slot());
                log::info!("Item: {} equipped? : {} : {}", item.get_name(), item.is_equipped(), item.get_equipment_slot().map_or_else(|| String::new(), |s| s.to_string()));
            }
        }
    }

    pub fn rebuild_to_container(&mut self, container: Container) {
        self.container = container;
        self.item_list_selection.cancel_selection();
        self.rebuild_selection();
    }

    // Callbacks return info on how to update the handler models
    pub fn handle_callback_result(&mut self, result: ContainerFrameHandlerInputResult) {
        match result {
            ContainerFrameHandlerInputResult::DropItems(undropped) => {
                self.retain_selected_items(undropped);
            },
            ContainerFrameHandlerInputResult::EquipItems(equipped) => {
                self.equip_items(equipped);
            },
            ContainerFrameHandlerInputResult::TakeItems(data) => {
                self.retain_selected_items(data.to_take);
            },
            ContainerFrameHandlerInputResult::MoveItems(data) => {
                // Moving into a container
                if let Some(_) = data.target_container {
                    self.container = data.source.clone();
                    self.item_list_selection.cancel_selection();
                    self.rebuild_selection();
                } else if let Some(_) = data.target_item {
                    // Moving to an existing item's location / splicing
                    // So just rebuild the whole selection
                    self.container = data.source.clone();
                    self.rebuild_selection_from(&data.source);
                }
            },
            ContainerFrameHandlerInputResult::MoveToContainerChoice(ref data) => {
                // Moving into a container
                if let Some(_) = data.target_container {
                    self.container = data.source.clone();
                    self.item_list_selection.cancel_selection();
                    self.rebuild_selection();
                }
            }
            _ => {}
        }
    }

    pub fn build_move_items_result(&self) -> Result<InputResult<ContainerFrameHandlerInputResult>, Error> {
        let from_container = self.container.clone();
        let selected_container_items = self.get_selected_items();
        let data = MoveToContainerChoiceData { source: from_container.clone(), to_move: selected_container_items, position: None, choices: Vec::new(), target_container: None };
        return Ok(InputResult {
            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
            view_specific_result: Some(ContainerFrameHandlerInputResult::MoveToContainerChoice(data))
        });
    }
}



impl <B : tui::backend::Backend> FrameHandler<B, &mut Container> for ContainerFrameHandler {

    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<&mut Container>) {
        let frame_size = data.get_frame_size().clone();
        let container = data.unpack();

        let window_block = Block::default()
            .borders(Borders::ALL)
            .title(container.get_self_item().get_name().clone());
        let window_area = Rect::new(frame_size.x.clone(), frame_size.y.clone(), frame_size.width.clone(), frame_size.height.clone());
        let inventory_item_lines = window_area.height - 3;
        self.row_count = inventory_item_lines as i32;
        self.item_list_selection.page_line_count = inventory_item_lines as i32;
        frame.render_widget(window_block, window_area);

        let headings = build_headings(self.columns.clone());
        let headings_area = Rect::new(frame_size.x.clone() + 1, frame_size.y.clone() + 1, frame_size.width.clone() - 4, 2);
        frame.render_widget(headings, headings_area);

        // -3 for the heading and 2  borders
        let mut line_index = 0;
        let start_index= self.item_list_selection.get_start_index();
        let end_of_page_representive_index = self.item_list_selection.get_end_of_page_index();

        if !container.get_contents().is_empty() {
            let view_contents = &container.get_contents()[start_index as usize..=end_of_page_representive_index as usize];
            for c in view_contents {
                let item_index = start_index.clone() + line_index.clone();
                let item = &c.get_self_item();
                let mut x_offset: u16 = frame_size.x.clone() as u16 + 1;
                let y_offset: u16 = frame_size.y.clone() as u16 + 2 + line_index.clone() as u16;
                let current_index = self.item_list_selection.is_focused(item_index);
                let selected = self.item_list_selection.is_selected(item_index);
                for column in &self.columns {
                    let text = build_column_text(column, item);
                    let mut column_text = build_paragraph(text);
                    if current_index.clone() && selected.clone() {
                        column_text = column_text.style(Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED));
                    } else if current_index {
                        column_text = column_text.style(Style::default().add_modifier(Modifier::REVERSED));
                    } else if selected {
                        column_text = column_text.style(Style::default().fg(Color::Green));
                    }

                    let column_length = column.size as i8;
                    let text_area = Rect::new(x_offset.clone(), y_offset.clone(), column_length.try_into().unwrap(), 1);
                    frame.render_widget(column_text.clone(), text_area);
                    x_offset += column_length as u16 + 1;
                }
                line_index += 1;
            }


            let usage_description = self.usage_line.describe();
            let usage_text = build_paragraph(usage_description.clone());
            let text_area = Rect::new( window_area.x.clone() + 1, window_area.y.clone() + window_area.height.clone() - 1, usage_description.len().try_into().unwrap(), 1);
            frame.render_widget(usage_text.clone(), text_area);

            // From right hand to left hand side draw the info text
            let page_count = build_page_count(&self.item_list_selection, window_area.clone());
            frame.render_widget(page_count.0, page_count.1);
            let page_count_text_length = page_count.2;
            let weight_limit = build_weight_limit(&self.container, window_area.clone(), page_count_text_length);
            frame.render_widget(weight_limit.0, weight_limit.1);
        }
    }
}

impl InputHandler<ContainerFrameHandlerInputResult> for ContainerFrameHandler {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<ContainerFrameHandlerInputResult>, Error> {
        let default_done_result = Ok(InputResult {
            generic_input_result: GenericInputResult { done: true, requires_view_refresh: true },
            view_specific_result: Some(ContainerFrameHandlerInputResult::None)});
        loop {
            let key = resolve_input(input)?;
            match key {
                Key::Char('d') => {
                    log::info!("[container frame handler] new result for DropItems..");
                    if self.usage_line.commands.contains_key( &Key::Char('d')) {
                        let selected_container_items = self.get_selected_items();
                        return Ok(InputResult {
                            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
                            view_specific_result: Some(ContainerFrameHandlerInputResult::DropItems(selected_container_items))
                        });
                    }
                },
                Key::Char('e') => {
                    if self.usage_line.commands.contains_key(&key) {
                        log::info!("[container frame handler] new result for EquipItems..");
                        let focused_item = self.find_focused_item().unwrap();
                        let mut items = Vec::new();
                        items.push(focused_item);
                        return Ok(InputResult {
                            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
                            view_specific_result: Some(ContainerFrameHandlerInputResult::EquipItems(items))
                        });
                    }
                },
                Key::Esc => {
                    if self.handle_quit()? {
                        return default_done_result;
                    }
                }
                Key::Char('o') => {
                    log::info!("[container frame handler] new result for OpenContainerView..");
                    if let Some(stacked_view) = self.build_view_for_focused_container() {
                        return Ok(InputResult {
                            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
                            view_specific_result: Some(ContainerFrameHandlerInputResult::OpenContainerView(stacked_view))
                        });
                    }
                },
                Key::Char('m') => {
                    return self.move_selected();
                },
                Key::Char('c') => {
                    return self.build_move_items_result();
                },
                Key::Char('\n') => {
                    self.toggle_select();
                },
                Key::Char(_c) => {},
                Key::Backspace => {},
                Key::Up => {
                    self.move_up();
                },
                Key::PageUp => {
                    self.page_up();
                },
                Key::Down => {
                    self.move_down();
                },
                Key::PageDown => {
                    self.page_down();
                },
                _ => {}
            }
            let continue_result = Ok(InputResult {
                generic_input_result: GenericInputResult { done: false, requires_view_refresh: false },
                view_specific_result: Some(ContainerFrameHandlerInputResult::None)});
            return continue_result;
        }
    }
}

pub fn build_default_container_view<'a>(container: Container) -> ContainerFrameHandler {
    let columns = build_default_columns();
    let items = container.to_cloned_item_list();
    ContainerFrameHandler {
        container: container.clone(),
        columns,
        row_count: 1,
        item_list_selection: build_list_selection(items, 1),
        usage_line : UsageLine::new(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::list_selection::{ListSelection};
    
    use crate::map::objects::container::{build, Container, ContainerType};
    use crate::map::objects::items;
    use crate::map::objects::items::Item;
    use crate::map::tile::Colour;
    use crate::menu;

    use crate::view::framehandler::container::{build_default_container_view, ContainerFrameHandler, ContainerFrameHandlerInputResult};

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container =  build(id, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container_self_item = container.get_self_item();
        assert_eq!(id, container_self_item.get_id());
        assert_eq!("Test Container", container_self_item.get_name());
        assert_eq!('X', container_self_item.symbol.character);
        assert_eq!(Colour::White, container_self_item.symbol.colour);
        assert_eq!(1, container_self_item.weight);
        assert_eq!(1, container_self_item.value);

        for i in 1..=4 {
            let test_item = Item::new(Uuid::new_v4(), format!("Test Item {}", i), 'X', 1, 100);
            container.add_item(test_item);
        }

        assert_eq!(ContainerType::OBJECT, container.container_type);
        assert_eq!(100, container.get_weight_limit());
        let contents = container.get_contents();
        assert_eq!(4, contents.len());
        container
    }

    #[test]
    fn test_view_build() {
        // GIVEN valid components
        let container = build_test_container();
        let _start_menu = menu::build_start_menu(false);
        // WHEN we call to build a new view
        let _view : ContainerFrameHandler = build_default_container_view(container);
        // THEN we expect to reach this point succesfully
    }

    #[test]
    fn test_move_focus_down() {
        // GIVEN a valid view
        let container = build_test_container();
        let mut view : ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;

        assert_eq!(0, view.item_list_selection.get_true_index());

        // WHEN we call to move down
        view.move_down();

        // THEN we expect the focused index to move
        assert_eq!(1, view.item_list_selection.get_true_index());
    }

    #[test]
    fn test_page_down() {
        // GIVEN a valid view
        let container = build_test_container();
        let mut view : ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;

        assert_eq!(0, view.item_list_selection.get_true_index());

        // WHEN we call to page down
        view.page_down();

        // THEN we expect the focused index to move to the end of the view / item count
        assert_eq!(3, view.item_list_selection.get_true_index());
    }

    #[test]
    fn test_move_focus_up() {
        // GIVEN a valid view
        let container = build_test_container();
        let _start_menu = menu::build_start_menu(false);
        let mut view : ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;

        assert_eq!(0, view.item_list_selection.get_true_index());

        // AND we've moved down a few times
        view.move_down();
        view.move_down();
        assert_eq!(2, view.item_list_selection.get_true_index());

        // WHEN we call to move up
        view.move_up();
        // THEN we expect the focused index to move
        assert_eq!(1, view.item_list_selection.get_true_index());
    }

    #[test]
    fn test_page_up() {
        // GIVEN a valid view
        let container = build_test_container();
        let mut view : ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;

        assert_eq!(0, view.item_list_selection.get_true_index());

        // AND we've already moved to the bottom of the view
        view.page_down();

        // WHEN we call to page up
        view.page_up();

        // THEN we expect the focused index to move to the start of the view
        assert_eq!(0, view.item_list_selection.get_true_index());
    }

    #[test]
    fn test_handle_callback_result_drop() {
        // GIVEN a valid view
        let container = build_test_container();
        let mut view: ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;
        assert_eq!(0, view.item_list_selection.get_true_index());
        let mut contents = view.container.get_contents_mut();
        assert_eq!(4, contents.len());
        // with a series of items
        assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[3].get_self_item().get_name());
        let retained_item = contents[1].get_self_item().clone();

        // AND we've selected the entire first page
        view.toggle_select();
        view.page_down();
        contents = &mut Vec::new();

        // WHEN we call to handle a DropItems callback with a retained item
        let result = ContainerFrameHandlerInputResult::DropItems(vec![retained_item]);
        view.handle_callback_result(result);
        // THEN we expect only the retained item to remain in the view container
        let contents = view.container.get_contents();
        assert_eq!(1, contents.len());
        assert_eq!("Test Item 2", contents[0].get_self_item().get_name());
    }

    #[test]
    fn test_handle_callback_result_take() {
        // GIVEN a valid view
        let container = build_test_container();
        let mut view: ContainerFrameHandler = build_default_container_view(container);
        view.item_list_selection.page_line_count = 4;
        assert_eq!(0, view.item_list_selection.get_true_index());
        let mut contents = view.container.get_contents_mut();
        assert_eq!(4, contents.len());
        // with a series of items
        assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[3].get_self_item().get_name());
        let retained_item = contents[1].get_self_item().clone();

        // AND we've selected the entire first page
        view.toggle_select();
        view.page_down();
        contents = &mut Vec::new();

        // WHEN we call to handle a DropItems callback with a retained item
        let result = ContainerFrameHandlerInputResult::DropItems(vec![retained_item]);
        view.handle_callback_result(result);
        // THEN we expect only the retained item to remain in the view container
        let contents = view.container.get_contents();
        assert_eq!(1, contents.len());
        assert_eq!("Test Item 2", contents[0].get_self_item().get_name());
    }
}