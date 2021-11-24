use crate::items::{Item, ItemType};

pub enum SelectionMode {
    SelectingItems
}

pub struct ListSelection<'a> {
    selection_mode : SelectionMode,
    previous_selection_index: i32,
    selection_index: i32,
    //The index of the topmost item on the screen, allows scrolling
    inv_start_index: i32,
    pivot_index: Option<i32>,
    previous_container_index: i32,
    container_index: i32,
    selecting_items: bool,
    // Storage of items in the selection
    selected_indices: Vec<i32>,
    selected_items: Vec<&'a Item>,
    items : Vec<&'a Item>,
    item_view_line_count: i32,
}

impl ListSelection<'_> {
    fn get_pivot_index(&self) -> i32 {
        match self.pivot_index {
            Some(idx) => { idx },
            None => { -1 }
        }
    }

    pub fn get_items(&self) -> Vec<&Item> {
        self.items.clone()
    }

    pub fn get_selected_items(&self) -> Vec<&Item> {
        self.selected_items.clone()
    }

    pub fn is_selecting(&self) -> bool {
        self.selecting_items.clone()
    }

    pub fn toggle_select(&mut self) {
        self.selecting_items = !self.selecting_items.clone();
    }

    pub fn is_selected(&self, index : i32) -> bool {
        self.selected_indices.contains(&index)
    }

    pub fn select(&mut self, index : i32) {
        if !self.is_selected(index) {
            let item_result = self.items.get(index.clone() as usize);
            match item_result {
                Some(item) => {
                    self.selected_indices.push(index.clone());
                    self.selected_items.push(item);
                },
                None => {}
            }
        }
    }

    pub fn select_range(&mut self, start : i32, end: i32) {
        for i in start..end {
            self.select(i);
        }
    }

    pub fn deselect(&mut self, index : i32) {
        if self.is_selected(index) {
            let item_result = self.items.get(index.clone() as usize);
            match item_result {
                Some(_) => {
                    self.selected_indices.remove(index.clone() as usize);
                    self.selected_items.remove(index.clone() as usize);
                },
                None => {}
            }
        }
    }

    pub fn deselect_range(&mut self, start : i32, end: i32) {
        for i in start..end {
            self.deselect(i);
        }
    }

    fn check_selecting_items_above(&mut self) {
        let selecting_items_above = self.container_index < self.previous_container_index && self.container_index < self.get_pivot_index();
        if selecting_items_above {
            for i in self.container_index..self.get_pivot_index() {
                self.select(i);
            }
        }
    }

    fn check_selecting_items_below(&mut self) {
        let selecting_items_below = self.container_index > self.previous_container_index && self.container_index > self.get_pivot_index();
        if selecting_items_below {
            for i in self.container_index..self.get_pivot_index() {
                self.select(i);
            }
        }
    }

    fn selected_start_position(&self) -> bool {
        self.container_index == self.get_pivot_index()
    }

    fn check_reducing_selection_above(&mut self) {
        // Deselect anything we were previously selecting
        let reducing_selection_above = self.container_index > self.previous_container_index && self.container_index < self.get_pivot_index();
        if reducing_selection_above || self.selected_start_position() {
            for i in self.previous_container_index.clone() - 1..self.container_index {
                self.deselect(i);
            }
        }
    }

    pub fn page_up(&mut self) -> i32 {
        let mut new_selection_index = self.selection_index;
        let mut max_selection_index = self.determine_max_selection_index();

        if self.selection_index == 0 && self.inv_start_index >= self.item_view_line_count {
            new_selection_index = max_selection_index;
            self.inv_start_index = self.inv_start_index - self.item_view_line_count;
            if self.selecting_items {
                self.select_range(new_selection_index, self.selection_index);
            }
            // TODO redraw list flag?
        } else if self.selection_index == 0 {
            new_selection_index = 0;
            self.inv_start_index = 0;
            // TODO redraw list flag?
        } else {
            new_selection_index = 0;
        }
        return new_selection_index.clone();
    }

    pub fn page_down(&mut self) -> i32 {
        let mut new_selection_index = self.selection_index;
        let max_scroll_index = self.determine_max_selection_index();
        let mut max_selection_index = self.determine_max_selection_index();

        if self.selection_index == max_selection_index && self.inv_start_index + self.item_view_line_count <= max_scroll_index {
            new_selection_index = 0;
            self.inv_start_index += self.item_view_line_count;
            // TODO redraw list flag?
        } else if self.selection_index == max_selection_index {
            self.inv_start_index = max_scroll_index;
            // TODO redraw list flag?
        } else {
            new_selection_index = max_selection_index;
        }
        new_selection_index.clone()
    }

    pub fn move_up(&mut self) -> i32 {
        if self.selection_index > 0 {
            return self.selection_index - 1;
        } else if self.selection_index == 0 && self.inv_start_index > 0 {
            self.inv_start_index -= 1;
            // TODO redraw list flag?
        }
        self.selection_index.clone()
    }

    pub fn move_down(&mut self) -> i32 {
        let max_scroll_index = self.determine_max_scroll_index();
        let max_selection_index = self.determine_max_selection_index();
        if self.selection_index < max_selection_index {
            return self.selection_index + 1;
        } else if self.selection_index == self.item_view_line_count-1 && self.inv_start_index < max_scroll_index {
            self.inv_start_index += 1;
            // TODO redraw list flag?
        }
        self.selection_index.clone()
    }

    pub fn determine_max_scroll_index(&mut self) -> i32 {
        let item_count = self.items.len() as i32;
        let mut max_scroll_index = 0;
        if item_count > self.item_view_line_count {
            max_scroll_index = item_count - self.item_view_line_count;
        }
        max_scroll_index
    }

    pub fn determine_max_selection_index(&mut self) -> i32 {
        let item_count = self.items.len() as i32;
        let mut max_selection_index = self.item_view_line_count-1;
        if item_count < self.item_view_line_count {
            let max =  self.item_view_line_count  - (self.item_view_line_count - item_count) - 1;
            max_selection_index = max.clone();
        }
        max_selection_index
    }

    fn check_reducing_selection_below(&mut self) {
        // Deselect anything we were previously selecting
        let reducing_selection_below = self.container_index < self.previous_container_index && self.container_index > self.get_pivot_index();
        if reducing_selection_below || self.selected_start_position() {
            for i in self.previous_container_index.clone() - 1..self.container_index {
                self.deselect(i);
            }
        }
    }

    pub fn update_selection_indices(&mut self, new_selection_index : i32) {
        self.previous_selection_index = self.selection_index;
        self.previous_container_index = self.container_index;
        self.selection_index = new_selection_index;
        self.container_index = new_selection_index.clone() + self.inv_start_index.clone();

        let mut no_pivot_point = false;
        match self.pivot_index {
            Some(_) => {},
            None => {
                no_pivot_point = true;
            }
        }

        if self.selecting_items && no_pivot_point {
            self.pivot_index = Some(new_selection_index.clone() + self.inv_start_index.clone());
        }

    }

    pub fn update_selection(&mut self, new_selection_index : i32) {
        self.update_selection_indices(new_selection_index);
        let selection_changed = self.container_index != self.previous_container_index;
        if selection_changed && self.selecting_items {
            self.check_selecting_items_above();
            self.check_reducing_selection_above();
            self.check_selecting_items_below();
            self.check_reducing_selection_below();
        }
    }

}

pub fn build_list_selection(items : Vec<&Item>, item_view_line_count: i32) -> ListSelection {
    let selection_mode = SelectionMode::SelectingItems;
    let previous_selection_index = 0;
    let selection_index = 0;
    let inv_start_index = 0;
    let pivot_index = None;
    let previous_container_index = 0;
    let container_index = 0;
    let selecting_items = false;
    let selected_indices = Vec::new();
    let selected_items = Vec::new();
    ListSelection { selection_mode, previous_selection_index, selection_index, inv_start_index, pivot_index, previous_container_index, container_index, selecting_items, selected_indices, selected_items, items, item_view_line_count }
}

#[cfg(test)]
mod tests {
    use crate::list_selection::{ListSelection, build_list_selection};

    #[test]
    fn test_build_list_selection() {
        // GIVEN a series of items to select from
        let item = crate::items::build_item(1, "Test Item 1".to_owned(), 'X', 1, 1);
        let item2 = crate::items::build_item(2, "Test Item 2".to_owned(), 'X', 1, 1);
        let item3 = crate::items::build_item(3, "Test Item 3".to_owned(), 'X', 1, 1);
        let item4 = crate::items::build_item(4, "Test Item 4".to_owned(), 'X', 1, 1);
        let items = vec! [ &item, &item2, &item3, &item4 ];

        // WHEN we call to build a list selection of these items
        let list_selection = build_list_selection(items, 4);

        // THEN we expect it to wrap the items provided
        let wrapped_items = list_selection.get_items();
        assert_eq!(4, wrapped_items.len());
        assert_eq!(item, *wrapped_items[0]);
        assert_eq!(item2, *wrapped_items[1]);
        assert_eq!(item3, *wrapped_items[2]);
        assert_eq!(item4, *wrapped_items[3]);

        // AND have no currently selected items
        let selected_items = list_selection.get_selected_items();
        assert_eq!(0, selected_items.len());
    }

    #[test]
    fn test_toggle_selection() {
        // GIVEN a list selection
        let mut list_selection = build_list_selection(Vec::new(), 0);
        // AND it's currently not selecting anything
        assert!(!list_selection.is_selecting());
        // WHEN we call to toggle selection
        list_selection.toggle_select();
        // THEN it will flip the is selecting state
        assert!(list_selection.is_selecting());
        // AND the opposite
        list_selection.toggle_select();
        assert!(!list_selection.is_selecting());
    }
}