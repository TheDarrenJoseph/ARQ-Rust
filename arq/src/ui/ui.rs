use std::convert::TryInto;
use termion::event::Key;
use std::io;
use std::io::{Error, ErrorKind};
use termion::input::TermRead;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, ListState, Paragraph, Widget, Wrap};

use crate::{menu, ui};
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::menu::{Menu, ToList};
use crate::ui::ui_areas::UIAreas;
use crate::ui::ui_util::{center_area, MIN_AREA};
use crate::view::framehandler::console::{ConsoleBuffer, ConsoleFrameHandler};
use crate::view::framehandler::{FrameData, FrameHandler};

use crate::widget::{StandardWidgetType, StatefulWidgetState, StatefulWidgetType};

pub struct UI {
    pub start_menu : Menu,
    pub render_additional: bool,
    pub console_visible: bool,
    pub additional_widgets: Vec<StandardWidgetType>,
    pub frame_size : Option<Area>,
    pub frame_handler: ConsoleFrameHandler
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
    UI { start_menu, frame_size : None, render_additional: false, console_visible: false, additional_widgets: Vec::new(), frame_handler }
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

    /*
        Tries to build a centered UIAreas based on 80x24 minimum frame size
        If the view is smaller than this, the view will be split as per usual for smaller sizes
     */
    pub fn get_min_area(&self, frame_size: Rect) -> UIAreas {
        if (frame_size.height >= 80 && frame_size.width >= 24) {
            let target = Rect::new(0, 0, 80, 24);
            let centered = center_area(target, frame_size, MIN_AREA).unwrap();
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(20)
                    ].as_ref()
                )
                .split(centered);

            UIAreas::new(areas[0], areas.get(1).cloned())
        } else {
            return self.get_view_areas(frame_size);
        }
    }


    // If the console if visible, splits a frame vertically into the 'main' and lower console areas
    // Otherwise returns the original frame size
    pub fn get_view_areas(&self, frame_size: Rect) -> UIAreas {
        let areas: Vec<Rect> = if self.console_visible {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(20)
                    ].as_ref()
                )
                .split(frame_size)
        } else {
            vec![frame_size.clone()]
        };
        UIAreas::new(areas[0], areas.get(1).cloned())
    }

    pub fn render<'a, B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        let main_block = build_main_block();
        let frame_size = frame.size();
        let areas: UIAreas = self.get_view_areas(frame_size);
        let main_area = areas.get_main_area();
        frame.render_widget(main_block, main_area.clone());

        let view_start_pos = Position { x : frame_size.x, y: frame_size.y };
        self.frame_size = Some(build_rectangular_area(view_start_pos, main_area.width, main_area.height ));

        if self.render_additional {
            self.draw_additional_widgets(frame);
        }

        if self.console_visible {
            self.draw_console(frame, areas.get_console_area().unwrap());
        }
    }

    pub fn show_console(&mut self) {
        self.console_visible = true;
    }

    pub fn hide_console(&mut self) {
        self.console_visible = false;
    }

    pub fn console_print(&mut self, input: String) {
        self.frame_handler.buffer.content = input;
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
        let frame_size = frame.size();
        let max_width = frame_size.width / 2;
        let widget_count = self.additional_widgets.len();
        let main_area = self.get_view_areas(frame_size).get_main_area();
        if widget_count > 0 {
            let mut _offset = 0;
            for widget in self.additional_widgets.iter_mut() {
                match widget {
                    StandardWidgetType::StatLine(w) => {
                        frame.render_widget(w.clone(),
                                            Rect::new(main_area.x + 1, main_area.y, max_width, 1));
                    },
                    StandardWidgetType::UsageLine(w) => {
                        frame.render_widget(w.clone(),
                                            Rect::new(main_area.x + 1, main_area.height - 1, main_area.width, 1));
                    },
                    _ => {}
                }
                _offset += 1;
            }
        }
    }
}

