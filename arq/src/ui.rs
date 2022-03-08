use tui::layout::{Alignment, Rect, Direction, Constraint, Layout};
use tui::style::{Color, Style};
use tui::text::{Spans,Span};
use tui::widgets::{Block, Borders, ListState, Paragraph, Wrap};

use std::convert::TryInto;

use crate::menu::{Menu, ToList};
use crate::widget::{Widget, WidgetType};
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::view::framehandler::console::{ConsoleFrameHandler, ConsoleBuffer};
use crate::{menu, ui};

pub struct UI {
    pub start_menu : Menu,
    pub settings_menu : Menu,
    pub render_additional: bool,
    pub console_visible: bool,
    pub additional_widgets: Vec<Widget>,
    pub frame_size : Option<Area>,
    pub console_view : ConsoleFrameHandler
}

pub enum StartMenuChoice {
    Play,
    Settings,
    Info,
    Quit
}

pub fn build_ui() -> UI {
    let start_menu = menu::build_start_menu(false);
    let settings_menu = menu::build_settings_menu();
    let console_view = ConsoleFrameHandler { buffer: ConsoleBuffer { content: String::from("") } };
    ui::UI { start_menu, settings_menu, frame_size : None, render_additional: false, console_visible: false, additional_widgets: Vec::new(), console_view :console_view }
}

// FrameHandlers are "dumb" views that simply draw themselves to a terminal frame
pub trait FrameHandler<B: tui::backend::Backend, T> {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<T>);
}

pub struct FrameData<T> {
    pub data : T,
    pub frame_size : Rect
}

impl <T> FrameData<T> {
    pub fn unpack(&mut self) -> &mut T {
        &mut self.data
    }
    pub fn get_frame_size(&mut self) -> &Rect {
        &self.frame_size
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
    fn draw_settings_menu<B : tui::backend::Backend>(&mut self, frame : &mut tui::terminal::Frame<'_, B>);
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
    pub fn get_view_areas(&self, frame_size: Rect) -> Vec<Rect> {
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
        areas
    }

    pub fn render<'a, B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        let main_block = build_main_block();
        let frame_size = frame.size();

        let areas: Vec<Rect> = self.get_view_areas(frame_size);
        let view_start_pos = Position { x : frame_size.x, y: frame_size.y };
        let main_area = areas[0];
        self.frame_size = Some(build_rectangular_area(view_start_pos, main_area.width, main_area.height ));
        frame.render_widget(main_block, areas[0]);

        if self.render_additional {
            self.draw_additional_widgets(frame);
        }

        if self.console_visible {
            self.draw_console(frame, areas[1]);
        }
    }

    pub fn show_console(&mut self) {
        self.console_visible = true;
    }

    pub fn hide_console(&mut self) {
        self.console_visible = false;
    }

    pub fn console_print(&mut self, input: String) {
        self.console_view.buffer.content = input;
    }
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

    fn draw_settings_menu<B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        self.render(frame);
        draw_menu(frame, &mut self.settings_menu);
    }

    fn draw_info<B: tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<'_, B>) {
        self.render(frame);
        let spans = vec![Spans::from(Span::raw("Made by Darren Joseph. Written in Rust."))];
        let spans_len = spans.len() as u16;
        let paragraph = Paragraph::new(spans)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let frame_size = frame.size();
        let paragraph_size = Rect::new(4, 4, frame_size.width / 2, spans_len + 2);
        frame.render_widget(paragraph, paragraph_size);
    }

    fn draw_console<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>, area: Rect) {
        let frame_data = FrameData { frame_size: area, data: ConsoleBuffer { content: self.console_view.buffer.content.clone() } };
        self.console_view.handle_frame(frame, frame_data);
    }

    fn draw_additional_widgets<B : tui::backend::Backend>(&mut self, frame: &mut tui::terminal::Frame<B>) {
        let frame_size = frame.size();
        let main_window_width = frame_size.width / 2;
        let widget_count = self.additional_widgets.len();
        if widget_count > 0 {
            let mut _offset = 0;
            for widget in self.additional_widgets.iter_mut() {
                match &mut widget.state_type {
                    WidgetType::StatLine(w) => {
                        let stats_area = Rect::new(1, 0, main_window_width, 1);
                        frame.render_stateful_widget(w.clone(), stats_area, &mut w.clone());
                    },
                    _ => {}
                }
                _offset += 1;
            }
        }
    }
}

