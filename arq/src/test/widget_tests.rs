use crate::widget::{Focusable, Widget, WidgetType, TextInputState, build_text_input, NumberInputState, DropdownInputState};

fn assert_for_text_widget<F>(widget_type : WidgetType, mut callback: F) where F : FnMut(TextInputState) {
    match widget_type {
        WidgetType::Text(s ) => {
           callback(s);
        }
        _ => {
            panic!("Widget state type was not Text!")
        }
    }
}

fn assert_for_number_widget<F>(widget_type : WidgetType, mut callback: F) where F : FnMut(NumberInputState) {
    match widget_type {
        WidgetType::Number(s ) => {
            callback(s);
        }
        _ => {
            panic!("Widget state type was not Number!")
        }
    }
}

fn assert_for_dropdown_widget<F>(widget_type : WidgetType, mut callback: F) where F : FnMut(DropdownInputState) {
    match widget_type {
        WidgetType::Dropdown(s ) => {
            callback(s);
        }
        _ => {
            panic!("Widget state type was not text!")
        }
    }
}

#[cfg(test)]
mod text_text_input {
    use crate::widget::{Focusable, Widget, WidgetType, TextInputState, build_text_input};
    use crate::test::widget_tests::assert_for_text_widget;

    #[test]
    fn test_text_input_add_char() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);
        assert_for_text_widget(text_input.state_type,  &|mut state: TextInputState| {
            // WHEN we add a character
            state.add_char('A');
            // THEN we expect the widget state input to be "A"
            assert_eq!("A".to_string(),  state.get_input());
        });
    }

    #[test]
    fn test_text_input_add_char_max_input() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);
        // WHEN we add 4 characters
        assert_for_text_widget(text_input.state_type,  &|mut state: TextInputState| {
            state.add_char('A');
            state.add_char('B');
            state.add_char('C');
            state.add_char('D');

            // THEN we expect the widget state input to be "ABC" and to have ignored the extra character
            assert_eq!("ABC".to_string(),  state.get_input());
        });
    }

    #[test]
    fn test_text_input_delete_char() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);
        assert_for_text_widget(text_input.state_type,  &|mut state: TextInputState| {
            // AND we've adjusted it's input to be "A"
            state.set_input("A".to_string());
            // WHEN we call to delete a char
            state.delete_char();
            // THEN we expect the widget state input to be ""
            assert_eq!("".to_string(),  state.get_input());
        });
    }

    #[test]
    fn test_text_input_delete_char_empty_field() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);
        assert_for_text_widget(text_input.state_type,  &|mut state: TextInputState| {
            // WHEN we call to delete a char
            state.delete_char();
            // THEN we expect the widget state input to be ""
            assert_eq!("".to_string(), state.get_input());
        });
    }

    #[test]
    fn test_text_input_delete_char_many() {
        // GIVEN a text input of 3 characters with no initial input
        let mut text_input = build_text_input(3, "Test".to_string(), 1);
        assert_for_text_widget(text_input.state_type,  &|mut state: TextInputState| {
                // AND we've adjusted it's input to be "ABC"
                state.set_input("ABC".to_string());
                // WHEN we call to delete 2 characters
                state.delete_char();
                state.delete_char();
                // THEN we expect the widget state input to be "A"
                assert_eq!("A".to_string(),  state.get_input());
        });
    }
}

#[cfg(test)]
mod text_number_input {
    use crate::widget::{Focusable, Widget, WidgetType, NumberInputState, build_number_input, build_number_input_with_value};
    use crate::test::widget_tests::assert_for_number_widget;

    #[test]
    fn test_build_number_input() {
        // GIVEN valid inputs
        // WHEN we call to build a number input
        let mut number_input = build_number_input(true,1, "A".to_string(), 1);
        assert_eq!(false, number_input.state_type.is_focused());
        assert_for_number_widget(number_input.state_type,  &|mut state: NumberInputState| {
            // THEN we expect it to be created without being focused
            assert_eq!(0, state.get_input());
        });
    }


    #[test]
    fn test_build_number_input_with_value() {
        // GIVEN valid inputs
        // WHEN we call to build a number input
        let mut number_input = build_number_input_with_value(true, 100,1, "A".to_string(), 1);
        assert_eq!(false, number_input.state_type.is_focused());
        assert_for_number_widget(number_input.state_type,  &|mut state: NumberInputState| {
            // THEN we expect it to be created without being focused
            assert_eq!(100, state.get_input());
        });
    }
}

#[cfg(test)]
mod text_dropdown {
    use crate::widget::{Focusable, Widget, WidgetType, DropdownInputState, build_dropdown};
    use crate::test::widget_tests::assert_for_dropdown_widget;

    #[test]
    fn test_dropdown_get_selection() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);
        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {
            // WHEN we call to get the initial selection
            // THEN we expect it to be "A"
            assert_eq!("A".to_string(),  state.get_selection());
            assert_eq!(false,  state.is_showing_options());
        });
    }

    #[test]
    fn test_dropdown_toggle_show() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {
                // WHEN we call to toggle showing of options
                state.toggle_show();
                // THEN we expect it to be set to true
                assert_eq!(true,  state.is_showing_options());
        });
    }

    #[test]
    fn test_dropdown_toggle_show_multi() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {

                // WHEN we call to toggle showing of options twice
                state.toggle_show();
                state.toggle_show();
                // THEN we expect it to be set to false again
                assert_eq!(false,  state.is_showing_options());
        });
    }


    #[test]
    fn test_dropdown_select_next() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);
        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {
            // WHEN we call to select the next item
            state.select_next();
            // THEN we expect the selection to be "B"
            assert_eq!("B".to_string(),  state.get_selection());
        });
    }

    #[test]
    fn test_dropdown_select_next_end_of_range() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {
            // WHEN we call to select the next item twice
            state.select_next();
            state.select_next();
            // THEN we expect the selection to be "B"
            assert_eq!("B".to_string(),  state.get_selection());
        });
    }

    #[test]
    fn test_dropdown_select_previous() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {
            // AND we've selected the 2nd option
            state.select_next();
            assert_eq!("B".to_string(),  state.get_selection());
            // WHEN we call to select the next item
            state.select_previous();
            // THEN we expect the selection to be "A" (unchanged)
            assert_eq!("A".to_string(),  state.get_selection());
        });
    }

    #[test]
    fn test_dropdown_select_previous_end_of_range() {
        // GIVEN a dropdown with 2 options
        let dropdown = build_dropdown("Test".to_string(), vec!["A".to_string(), "B".to_string()]);

        assert_for_dropdown_widget(dropdown.state_type,  &|mut state: DropdownInputState| {
            assert_eq!("A".to_string(),  state.get_selection());
            // WHEN we call to select the next item
            state.select_previous();
            // THEN we expect the selection to be "A" (unchanged)
            assert_eq!("A".to_string(),  state.get_selection());
        });
    }

}
