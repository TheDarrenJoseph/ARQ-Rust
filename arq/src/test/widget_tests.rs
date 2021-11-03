#[cfg(test)]
mod text_text_input {
    use crate::widget::{Focusable, Widget, WidgetType, TextInputState, build_text_input};

    #[test]
    fn test_text_input_add_char() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        // WHEN we add a character
        match text_input.state_type {
            WidgetType::Text(mut state) => {
                state.add_char('A');

                // THEN we expect the widget state input to be "A"
                assert_eq!("A".to_string(),  state.get_input());
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_add_char_max_input() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        // WHEN we add 4 characters
        match text_input.state_type {
            WidgetType::Text(mut state) => {
                state.add_char('A');
                state.add_char('B');
                state.add_char('C');
                state.add_char('D');

                // THEN we expect the widget state input to be "ABC" and to have ignored the extra character
                assert_eq!("ABC".to_string(),  state.get_input());
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_delete_char() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        match text_input.state_type {
            WidgetType::Text(mut state) => {
                // AND we've adjusted it's input to be "A"
                state.set_input("A".to_string());
                // WHEN we call to delete a char
                state.delete_char();
                // THEN we expect the widget state input to be ""
                assert_eq!("".to_string(),  state.get_input());
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_delete_char_empty_field() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        match text_input.state_type {
            WidgetType::Text(mut state) => {
                // WHEN we call to delete a char
                state.delete_char();
                // THEN we expect the widget state input to be ""
                assert_eq!("".to_string(),  state.get_input());
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }

    #[test]
    fn test_text_input_delete_char_many() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);

        match text_input.state_type {
            WidgetType::Text(mut state) => {
                // AND we've adjusted it's input to be "ABC"
                state.set_input("ABC".to_string());
                // WHEN we call to delete 2 characters
                state.delete_char();
                state.delete_char();
                // THEN we expect the widget state input to be "A"
                assert_eq!("A".to_string(),  state.get_input());
            },
            _ => {
                panic!("Widget state type was not text!")
            }
        }
    }
}

#[cfg(test)]
mod text_dropdown {
    use crate::widget::{Focusable, Widget, WidgetType, DropdownInputState, build_dropdown, build_text_input};

    #[test]
    fn test_dropdown_get_selection() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to get the initial selection
                // THEN we expect it to be "A"
                assert_eq!("A".to_string(),  state.get_selection());
                assert_eq!(false,  state.is_showing_options());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_toggle_show() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to toggle showing of options
                state.toggle_show();
                // THEN we expect it to be set to true
                assert_eq!(true,  state.is_showing_options());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_toggle_show_multi() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to toggle showing of options twice
                state.toggle_show();
                state.toggle_show();
                // THEN we expect it to be set to false again
                assert_eq!(false,  state.is_showing_options());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }


    #[test]
    fn test_dropdown_select_next() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to select the next item
                state.select_next();
                // THEN we expect the selection to be "B"
                assert_eq!("B".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_select_next_end_of_range() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // WHEN we call to select the next item twice
                state.select_next();
                state.select_next();
                // THEN we expect the selection to be "B"
                assert_eq!("B".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_select_previous() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                // AND we've selected the 2nd option
                state.select_next();
                assert_eq!("B".to_string(),  state.get_selection());
                // WHEN we call to select the next item
                state.select_previous();
                // THEN we expect the selection to be "A" (unchanged)
                assert_eq!("A".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

    #[test]
    fn test_dropdown_select_previous_end_of_range() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        match dropdown.state_type {
            WidgetType::Dropdown(mut state) => {
                assert_eq!("A".to_string(),  state.get_selection());
                // WHEN we call to select the next item
                state.select_previous();
                // THEN we expect the selection to be "A" (unchanged)
                assert_eq!("A".to_string(),  state.get_selection());
            },
            _ => {
                panic!("Widget state type was not Dropdown!")
            }
        }
    }

}
