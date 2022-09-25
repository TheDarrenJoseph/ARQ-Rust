use termion::event::Key;
use crate::map::position::Area;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::UI;
use crate::view::{GenericInputResult, InputHandler, InputResult, resolve_input, View};
use crate::widget::{Focusable, WidgetType};
use crate::widget::widgets::WidgetList;

pub struct WidgetMenu {
    pub selected_widget: Option<i8>,
    pub widgets: WidgetList
}
