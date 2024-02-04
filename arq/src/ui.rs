use std::error::Error;
use std::io;
use crate::build_paragraph;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::ui::UI;
use crate::ui::ui_util::check_display_size;

pub mod ui;
pub mod ui_wrapper;
pub mod ui_layout;
pub mod ui_areas;
pub mod ui_areas_builder;
pub mod ui_util;
