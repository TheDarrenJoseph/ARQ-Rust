use std::io;
use std::io::Error;
use std::convert::TryInto;
use std::collections::VecDeque;

use tui::layout::{Alignment, Rect};
use tui::style::{Style, Color, Modifier};
use tui::buffer::{Buffer};
use tui::widgets::{Block, Borders, ListState, Paragraph, Wrap};
use tui::text::{Spans,Span};
use termion::input::TermRead;
use termion::event::Key;

use crate::ui::{UI, FrameHandler, FrameData};
use crate::view::{View, resolve_input, resolve_area, InputHandler, InputResult, GenericInputResult};
use crate::terminal::terminal_manager::TerminalManager;
use crate::map::objects::container::Container;
use crate::map::objects::items::Item;
use crate::list_selection::{ListSelection, ItemListSelection, build_list_selection};
use crate::map::position::Area;

pub struct ContainerView {
    pub container : Container,
    columns : Vec<Column>,
    row_count: i32,
    pub item_list_selection : ItemListSelection
}

pub enum ContainerViewInputResult {
    NONE,
    OPEN_CONTAINER_VIEW(ContainerView)
}

pub fn build_container_view(container: Container) -> ContainerView {
    let columns = vec![
        Column {name : "NAME".to_string(), size: 12},
        Column {name : "WEIGHT (Kg)".to_string(), size: 12},
        Column {name : "VALUE".to_string(), size: 12}
    ];

    let mut items = Vec::new();
    // Clone the self items for everything in the container
    for c in container.get_contents() {
        items.push(c.get_self_item().clone());
    }
    ContainerView { container: container.clone(),
            columns,
            row_count: 1,
            item_list_selection: build_list_selection(items, 1)
    }
}

impl ContainerView {

    fn build_headings(&self) -> Paragraph {
        let mut heading_spans = Vec::new();
        let mut spans = Vec::new();
        for column in &self.columns {
            let name = column.name.clone();
            let padding = build_padding(column.size - name.len() as i8 + 2);
            spans.push(Span::raw(column.name.clone()));
            spans.push(Span::raw(padding));
        }
        heading_spans.push(Spans::from(spans));
        Paragraph::new(heading_spans)
            .block(Block::default()
                .borders(Borders::NONE))
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
    }

    pub fn rebuild_selection(&mut self, container: &Container) {
        let mut items = Vec::new();
        // Clone the self items for everything in the container
        for c in container.get_contents() {
            items.push(c.get_self_item().clone());
        }
        self.item_list_selection = build_list_selection(items, 1);
    }

    fn get_selected_items(&self) -> Vec<Item> {
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

    fn open_focused(&mut self) -> Option<ContainerView> {
        if !self.item_list_selection.is_selecting() {
            if let Some(focused_item) = self.item_list_selection.get_focused_item() {
                if focused_item.is_container() {
                    if let Some(focused_container) = self.container.find_mut(focused_item) {
                        return Some(build_container_view(focused_container.clone()))
                    }
                }
            }
        }
        None
    }

    fn move_selection(&mut self) {
        let list_selection = &self.item_list_selection;
        let selected_container_items = &self.get_selected_containers();
        let mut updated = false;
        if list_selection.is_selecting() {
            let focused_item_result = list_selection.get_focused_item();
            let true_index = list_selection.get_true_index();
            if let Some(focused_item) = focused_item_result {
                // Make sure we've not focused any of the selected items
                let focused_items = selected_container_items.iter().find(|ci| ci.get_self_item().get_id() == focused_item.get_id());
                if let None = focused_items {
                    if focused_item.is_container() {
                        if let Some(focused_container) = self.container.find_mut(focused_item) {
                            // Move items into the container
                            focused_container.push(selected_container_items.clone());
                            self.container.remove(selected_container_items.to_vec());
                        }
                    } else {
                        // Move items to this location
                        self.container.remove(selected_container_items.to_vec());
                        self.container.insert(true_index as usize - selected_container_items.len(), selected_container_items.clone());
                    }
                    &mut self.item_list_selection.cancel_selection();
                    updated = true;
                }
            }
        }

        if updated {
            self.rebuild_selection(&self.container.clone());
        }
    }
}

pub struct Column {
    pub name : String,
    pub size : i8
}

fn build_padding(length : i8) -> String {
    let mut s = String::new();
    for i in 1..length {
        s.push(' ');
    }
    s
}


fn build_column_text(column: &Column, item: &Item) -> String {
    match column.name.as_str() {
        "NAME" => {
            item.get_name()
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

fn build_paragraph<'a>(text: String) -> Paragraph<'a> {
    let spans = vec![Spans::from(Span::raw(text.clone()))];
    let spans_len = spans.len() as u16;
    let paragraph = Paragraph::new(spans)
        .style(Style::default())
        .alignment(Alignment::Left);
    paragraph
}

fn get_row_count(frame_height: i32, container_len: i32) -> i32 {
    let available_frame_rows = frame_height - 2;
    if container_len < available_frame_rows {
        container_len
    } else {
        available_frame_rows
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, &mut Container> for ContainerView {

    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<&mut Container>) {
        let frame_size = data.get_frame_size().clone();
        let container = data.unpack();
        let container_len = container.get_contents().len() as i32;

        let window_block = Block::default()
            .borders(Borders::ALL)
            .title(container.get_self_item().name.clone());
        let window_area = Rect::new(frame_size.x.clone(), frame_size.y.clone(), frame_size.width.clone() - 4, frame_size.height.clone() - 4);
        let inventory_item_lines = window_area.height - 3;
        self.row_count = inventory_item_lines as i32;
        self.item_list_selection.page_line_count = inventory_item_lines as i32;
        frame.render_widget(window_block, window_area);

        let headings = self.build_headings();
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
                let mut y_offset: u16 = frame_size.y.clone() as u16 + 2 + line_index.clone() as u16;
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

            let usage_description = "(o)pen, (d)rop, (m)ove";
            let mut usage_text = build_paragraph(String::from(usage_description));
            let text_area = Rect::new( window_area.x.clone() + 1, window_area.y.clone() + window_area.height.clone() - 1, usage_description.len().try_into().unwrap(), 1);
            frame.render_widget(usage_text.clone(), text_area);


            let page_number = self.item_list_selection.get_page_number();
            let total_pages = self.item_list_selection.get_total_pages();

            let page_count_text = format!("Page {}/{}", page_number, total_pages);
            let page_count_text_length = page_count_text.len();
            let width = page_count_text.len().try_into().unwrap();
            let page_count_paragraph = build_paragraph(page_count_text);
            let page_count_area = Rect::new( window_area.width.clone() - page_count_text_length as u16 , window_area.y.clone() + window_area.height.clone() - 1, width, 1);
            frame.render_widget(page_count_paragraph, page_count_area);
        }
    }
}

impl InputHandler<ContainerViewInputResult> for ContainerView {
    fn handle_input(&mut self, input: Option<Key>) -> Result<InputResult<ContainerViewInputResult>, Error> {
        let default_done_result = Ok(InputResult {
            generic_input_result: GenericInputResult { done: true, requires_view_refresh: true },
            view_specific_result: Some(ContainerViewInputResult::NONE)});
        loop {
            let key = resolve_input(input);
            match key {
                Key::Char('q') => {
                    if self.handle_quit()? {
                        return default_done_result;
                    }
                },
                Key::Char('o') => {
                    if let Some(stacked_view) = self.open_focused() {
                        let new_view_result = Ok(InputResult {
                            generic_input_result: GenericInputResult { done: false, requires_view_refresh: true },
                            view_specific_result: Some(ContainerViewInputResult::OPEN_CONTAINER_VIEW(stacked_view))
                        });
                        return new_view_result;
                    }
                },
                Key::Char('m') => {
                    self.move_selection();
                },
                Key::Char('\n') => {
                    self.toggle_select();
                },
                Key::Char(c) => {},
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
                view_specific_result: Some(ContainerViewInputResult::NONE)});
            return continue_result;
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use tui::backend::TestBackend;

    use crate::ui;
    use crate::terminal;
    use crate::map::objects::container;
    use crate::map::objects::container::{build, ContainerType, Container};
    use crate::map::objects::items;
    use crate::menu;
    use crate::view::container_view::{ContainerView, build_container_view};
    use crate::terminal::terminal_manager::TerminalManager;
    use crate::ui::{UI, build_ui};
    use crate::list_selection::ListSelection;
    use crate::view::console_view::{ConsoleView, ConsoleBuffer};

    fn build_test_container() -> Container {
        let id = Uuid::new_v4();
        let mut container =  build(id, "Test Container".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 100);
        let container_self_item = container.get_self_item();
        assert_eq!(id, container_self_item.get_id());
        assert_eq!("Test Container", container_self_item.name);
        assert_eq!('X', container_self_item.symbol);
        assert_eq!(0, container_self_item.colour);
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

    #[test]
    fn test_view_build() {
        // GIVEN valid components
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        let start_menu = menu::build_start_menu(false);
        let settings_menu = menu::build_settings_menu();
        // WHEN we call to build a new view
        let view : ContainerView = build_container_view(container);
        // THEN we expect to reach this point succesfully
    }

    #[test]
    fn test_move_focus_down() {
        // GIVEN a valid view
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        let mut view : ContainerView = build_container_view(container);
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
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        let mut view : ContainerView = build_container_view(container);
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
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        let start_menu = menu::build_start_menu(false);
        let settings_menu = menu::build_settings_menu();
        let mut view : ContainerView = build_container_view(container);
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
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        let mut view : ContainerView = build_container_view(container);
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
    fn test_move_items() {
        // GIVEN a valid view
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        let mut view : ContainerView = build_container_view(container);
        view.item_list_selection.page_line_count = 4;
        assert_eq!(0, view.item_list_selection.get_true_index());
        let mut contents = view.container.get_contents();
        assert_eq!(4, contents.len());
        // with a series of items
        assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[3].get_self_item().get_name());

        // AND we've started selecting items
        view.toggle_select();
        // AND we've selected the first 2 items
        view.move_down();
        view.toggle_select();

        // WHEN we move to the bottom of the view and try to move the items
        view.page_down();
        view.move_selection();

        // THEN we expect the focused index to remain at the top of the view
        assert_eq!(0, view.item_list_selection.get_true_index());

        // AND the chosen items will be moved to the bottom of the view above the last item
        let contents = view.container.get_contents();
        assert_eq!(4, contents.len());
        assert_eq!("Test Item 3", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 1", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[3].get_self_item().get_name());
    }

    #[test]
    fn test_move_items_into_container() {
        // GIVEN a valid view
        let mut terminal_manager : &mut TerminalManager<TestBackend> = &mut terminal::terminal_manager::init_test().unwrap();
        let mut container = build_test_container();
        // With a main container including items and another container
        let mut bag =  build(Uuid::new_v4(), "Bag".to_owned(), 'X', 1, 1,  ContainerType::OBJECT, 4);
        container.add(bag);

        let mut view : ContainerView = build_container_view(container);
        view.item_list_selection.page_line_count = 5;
        assert_eq!(0, view.item_list_selection.get_true_index());
        let mut contents = view.container.get_contents();
        assert_eq!(5, contents.len());
        assert_eq!("Test Item 1", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 2", contents[1].get_self_item().get_name());
        assert_eq!("Test Item 3", contents[2].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[3].get_self_item().get_name());
        assert_eq!("Bag", contents[4].get_self_item().get_name());

        view.toggle_select();
        // AND we've selected the first 2 items
        view.move_down();
        view.toggle_select();

        // WHEN we move to the bottom of the view and try to move the items into the bag
        view.page_down();
        view.move_selection();

        // THEN the chosen items will be moved into the bag
        assert_eq!(0, view.item_list_selection.get_true_index());
        let contents = view.container.get_contents();
        assert_eq!(3, contents.len());
        assert_eq!("Test Item 3", contents[0].get_self_item().get_name());
        assert_eq!("Test Item 4", contents[1].get_self_item().get_name());
        assert_eq!("Bag",         contents[2].get_self_item().get_name());

        let bag_contents = contents[2].get_contents();
        assert_eq!(2, bag_contents.len());
        assert_eq!("Test Item 1", bag_contents[0].get_self_item().get_name());
        assert_eq!("Test Item 2", bag_contents[1].get_self_item().get_name());
    }
}