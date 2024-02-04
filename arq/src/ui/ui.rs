use std::convert::TryInto;
use termion::event::Key;
use std::io;

use termion::input::TermRead;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, ListState, Paragraph, Widget, Wrap};

use crate::{menu};
use crate::map::position::{Area};
use crate::menu::{Menu, ToList};
use crate::ui::ui_areas::{UI_AREA_NAME_CONSOLE, UI_AREA_NAME_MAIN, UIAreas};
use crate::ui::ui_areas_builder::UIAreasBuilder;
use crate::ui::ui_layout::UILayout;
use crate::ui::ui_util::{center_area, MIN_AREA};
use crate::view::framehandler::console::{ConsoleBuffer, ConsoleFrameHandler};
use crate::view::framehandler::{FrameData, FrameHandler};

use crate::widget::{StandardWidgetType};

pub struct UI {
    start_menu : Menu,
    pub render_additional: bool,
    console_visible: bool,
    additional_widgets: Vec<StandardWidgetType>,
    frame_size : Option<Area>,
    frame_handler: ConsoleFrameHandler,
    pub ui_layout: Option<UILayout>
}

#[derive(Clone)]
pub enum StartMenuChoice {
    Play,
    Settings,
    Info,
    Quit
}

pub fn build_ui() -> UI {
    let start_menu = menu::build_start_menu(false);
    let frame_handler = ConsoleFrameHandler { buffer: ConsoleBuffer { content: String::from("") } };
    UI {
        start_menu,
        frame_size : None,
        render_additional: false,
        console_visible: false,
        additional_widgets: Vec::new(),
        frame_handler,
        ui_layout: None
    }
}

impl std::convert::TryFrom<usize> for StartMenuChoice {
    type Error = String;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(StartMenuChoice::Play),
            1 => Ok(StartMenuChoice::Settings),
            2 => Ok(StartMenuChoice::Info),
            3 =>  Ok(StartMenuChoice::Quit),
            _ => Err("Failed to convert to StartMenuChoice".to_string())
        }
    }
}

pub enum SettingsMenuChoice {
    FogOfWar,
    Quit
}

impl std::convert::TryFrom<usize> for SettingsMenuChoice {
    type Error = String;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(SettingsMenuChoice::FogOfWar),
            1 => Ok(SettingsMenuChoice::Quit),
            _ => Err("Failed to convert to StartMenuChoice".to_string())
        }
    }
}


pub trait Draw {
    fn draw_start_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>);
    fn draw_info<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>);
    fn draw_console<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>, area: Rect);
    fn draw_additional_widgets<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>);
}

fn build_main_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        //.title("||ARQ -- ASCII Roguelike Quester||")
        .style(Style::default().bg(Color::Black))
}

impl UI {
    pub fn rebuild_start_menu(&mut self, game_started: bool) {
        self.start_menu = menu::build_start_menu(game_started);
    }

    /*
        Draws the "base" UI that's visible by default:
        1. The "main" window / main block (This does not put content inside this block, it's just an empty bordered window)
        2. The console window at the bottom of the screen
        3. Stat bars and command usage hints (additional widgets)
     */
    pub fn render<'a, B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        let frame_size = Area::from_rect(frame.size());
        self.frame_size = Some(frame_size);
        if self.ui_layout.is_none() {
            self.ui_layout = Some(UILayout::new(frame_size));
        }

        let ui_layout = self.ui_layout.as_mut().unwrap();
        let areas = ui_layout.get_or_build_areas(frame.size());

        if let Some(main) = areas.get_area(UI_AREA_NAME_MAIN) {
            let main_area = main.area;
            let main_block = build_main_block();
            {
                frame.render_widget(main_block, main_area);
            }
        }

        if let Some(console) = areas.get_area(UI_AREA_NAME_CONSOLE) {
            let console_area = console.area;
            if self.console_visible {
                self.draw_console(frame, console_area);
            }
        }

        if self.render_additional {
            self.draw_additional_widgets(frame);
        }
    }

    pub fn show_console(&mut self) {
        self.console_visible = true;
    }

    pub fn hide_console(&mut self) {
        self.console_visible = false;
    }

    /*
        Updates the buffer that is written to console on each rendering of the UI
     */
    pub fn set_console_buffer(&mut self, input: String) {
        self.frame_handler.buffer.content = input;
    }

    pub fn clear_console_buffer(&mut self) {
        self.frame_handler.buffer.content = String::new();
    }

    pub fn get_start_menu(&self) -> &Menu {
        &self.start_menu
    }

    pub fn get_start_menu_mut(&mut self) -> &mut Menu {
        &mut self.start_menu
    }

    pub fn is_console_visible(&self) -> bool {
        self.console_visible
    }

    pub fn get_additional_widgets(&self) -> &Vec<StandardWidgetType> {
        &self.additional_widgets
    }

    pub fn get_additional_widgets_mut(&mut self) -> &mut Vec<StandardWidgetType> {
        &mut self.additional_widgets
    }
}

pub fn get_input_key() -> Result<Key, io::Error> {
    Ok(io::stdin().keys().next().unwrap()?)
}

fn draw_menu<B: tui::backend::Backend>(frame: &mut tui::terminal::Frame<'_, B>, menu : &mut  Menu) {
    let mut menu_list_state = ListState::default();
    menu_list_state.select(Some(menu.selection.try_into().unwrap()));

    let frame_size = frame.size();
    let menu_size = Rect::new(4, 4, frame_size.width / 2, menu.menu_titles.len().try_into().unwrap());
    let menu_list = menu.to_list();
    frame.render_stateful_widget(menu_list, menu_size, &mut menu_list_state);
}

impl Draw for UI {

    fn draw_start_menu<B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        self.render(frame);
        draw_menu(frame, &mut self.start_menu);
    }

    fn draw_info<B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        self.render(frame);

        let dev_spans = vec!(Span::raw("Made by Darren Joseph. Written in Rust."));
        let spans = vec![Spans::from(dev_spans),
                         Spans::from(Span::raw("--- Credits ---")),
                         Spans::from(Span::raw("Background music:")),
                         Spans::from(Span::raw("Tavern Loop One by Alexander Nakarada | https://www.serpentsoundstudios.com")),
                         Spans::from(Span::raw("Music promoted by https://www.free-stock-music.com")),
                         Spans::from(Span::raw("Attribution 4.0 International (CC BY 4.0)")),
                         Spans::from(Span::raw("https://creativecommons.org/licenses/by/4.0/"))
        ];

        let spans_len = spans.len() as u16;
        let paragraph = Paragraph::new(spans)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let frame_size = frame.size();
        let paragraph_size = Rect::new(4, 4, frame_size.width - 4, spans_len + 4);
        frame.render_widget(paragraph, paragraph_size);
    }

    fn draw_console<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>, area: Rect) {
        let frame_data = FrameData { frame_size: area, data: ConsoleBuffer { content: self.frame_handler.buffer.content.clone() } };
        self.frame_handler.handle_frame(frame, frame_data);
    }

    fn draw_additional_widgets<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>) {
        let widget_count = self.additional_widgets.len();
        if let Some(main_area) = self.ui_layout.as_ref().unwrap().get_ui_areas().get_area(UI_AREA_NAME_MAIN) {
            let area = main_area.area;
            let max_width = area.width / 2;
            if widget_count > 0 {
                let mut _offset = 0;
                for widget in self.additional_widgets.iter_mut() {
                    match widget {
                        StandardWidgetType::StatLine(w) => {
                            frame.render_widget(w.clone(),
                                                Rect::new(area.x + 1, area.y, max_width, 1));
                        },
                        StandardWidgetType::UsageLine(w) => {
                            frame.render_widget(w.clone(),
                                                Rect::new(area.x + 1, area.height - 1, area.width, 1));
                        },
                        _ => {}
                    }
                    _offset += 1;
                }
            }
        }
    }
}


