use crate::settings::Settings;
use crate::ui::resolution::Resolution;
use crate::widget::{Focusable, StatefulWidgetState};
use crate::widget::stateful::boolean_widget::build_boolean_widget;
use crate::widget::stateful::dropdown_widget::{build_dropdown, DropdownOption};
use crate::widget::stateful::number_widget::build_number_input_with_value;
use crate::widget::stateful::text_widget::build_text_input;

pub struct WidgetList {
    pub widget_index: Option<i8>,
    pub widgets: Vec<StatefulWidgetState>
}

impl WidgetList {
    pub fn previous_widget(&mut self) {
        let selected_widget = self.widget_index.unwrap();
        if selected_widget > 0 && selected_widget < self.widgets.len() as i8 {
            self.select_widget(selected_widget - 1);
        }
    }

    pub fn next_widget(&mut self) {
        let selected_widget = self.widget_index.unwrap();
        if selected_widget >= 0 && selected_widget < self.widgets.len() as i8 - 1 {
            self.select_widget(selected_widget + 1);
        }
    }

    pub fn select_widget(&mut self, index: i8) {
        let mut offset = 0;
        for widget in self.widgets.iter_mut() {
            if offset == index {
                self.widget_index =  Some(offset.clone());
                widget.state_type.focus();
            } else {
                widget.state_type.unfocus();
            }
            offset += 1;
        }
    }
}

pub fn build_settings_widgets(settings : &Settings) -> Vec<StatefulWidgetState> {
    let mut widgets = Vec::new();
    for setting in &settings.bool_settings {
        widgets.push(build_boolean_widget(15, setting.name.clone(), setting.value))
    }

    for setting in &settings.string_settings {
        widgets.push(build_text_input(15, setting.name.clone(), setting.value.clone(), 1))
    }
    for setting in &settings.u32_settings {
        widgets.push(build_number_input_with_value(true, setting.value.clone() as i32,15, setting.name.clone(), 1));
    }

    for setting in &settings.dropdown_settings {
        let mut options : Vec<String> = Vec::new();

        let chosen_option_name = String::from(setting.value.chosen_option.display_name);
        options.push(chosen_option_name.clone());
        let other_options : Vec<DropdownOption<Resolution>> = setting.value.options.iter().filter(|o| String::from(o.display_name) != chosen_option_name ).map(|o| o.clone()).collect();
        for option in &other_options {
            options.push(String::from(option.display_name))
        }
        let dropdown = build_dropdown(setting.name.clone(), true, options);
        widgets.push(dropdown)
    }
    widgets
}