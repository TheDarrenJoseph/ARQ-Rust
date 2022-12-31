use crate::settings::Settings;
use crate::widget::{Focusable, StatefulWidgetState};
use crate::widget::boolean_widget::build_boolean_widget;
use crate::widget::number_widget::build_number_input_with_value;
use crate::widget::text_widget::build_text_input;

pub struct WidgetList {
    pub selected_widget: Option<i8>,
    pub widgets: Vec<StatefulWidgetState>
}

impl WidgetList {
    pub fn previous_widget(&mut self) {
        let selected_widget = self.selected_widget.unwrap();
        if selected_widget > 0 && selected_widget < self.widgets.len() as i8 {
            self.select_widget(selected_widget - 1);
        }
    }

    pub fn next_widget(&mut self) {
        let selected_widget = self.selected_widget.unwrap();
        if selected_widget >= 0 && selected_widget < self.widgets.len() as i8 - 1 {
            self.select_widget(selected_widget + 1);
        }
    }

    pub fn select_widget(&mut self, index: i8) {
        let mut offset = 0;
        for widget in self.widgets.iter_mut() {
            if offset == index {
                self.selected_widget =  Some(offset.clone());
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

    widgets
}