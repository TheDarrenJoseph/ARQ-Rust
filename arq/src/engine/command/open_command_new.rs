use std::io;
use log::info;
use termion::event::Key;
use tokio::sync::mpsc;
use crate::engine::command::command::Command;
use crate::engine::command::open_command::OpenCommand;
use crate::engine::level::Level;
use crate::engine::message::channels::MessageChannels;
use crate::error::errors::ErrorWrapper;
use crate::input::{IoKeyInputResolver, KeyInputResolver, MockKeyInputResolver};
use crate::item_list_selection::{ItemListSelection, ListSelection};
use crate::map::objects::container::Container;
use crate::map::position::Position;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::bindings::input_bindings::KeyBindings;
use crate::ui::bindings::open_bindings::{map_open_input_to_side, OpenInput, OpenKeyBindings};
use crate::ui::event::{Event, EventHandler, EventTask};
use crate::ui::ui::{get_input_key, UI};
use crate::ui::ui_layout::LayoutType;
use crate::ui::ui_layout::LayoutType::StandardSplit;
use crate::view::framehandler::container;
use crate::view::framehandler::util::tabling::Column;
use crate::view::InputResult;
use crate::view::model::usage_line::{UsageCommand, UsageLine};
use crate::view::world_container_view::{WorldContainerView, WorldContainerViewFrameHandlers};
use crate::widget::stateful::container_widget::{ContainerWidget, ContainerWidgetData};
use crate::widget::{Named, StatefulWidgetType};

pub struct OpenCommandNew<'a, B: 'static + ratatui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub input_resolver: Box<dyn KeyInputResolver>,
    pub key_bindings: OpenKeyBindings
}

const UI_USAGE_HINT: &str = "Up/Down - Move\nEnter/q - Toggle/clear selection\nEsc - Exit";
const NOTHING_ERROR : &str = "There's nothing here to open.";

pub enum OpenContainerEvent {
    MoveUp,
    MoveDown,
    ToggleSelection
}

impl <B: ratatui::backend::Backend> OpenCommandNew<'_, B> {

    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        let level = self.level.clone();
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(Some(level), None, frame);
        })?;
        Ok(())
    }
    
    fn get_input_resolver(&mut self) -> Box<dyn KeyInputResolver> {
        let mock_input_resolver = &mut self.input_resolver.as_any_mut().downcast_mut::<MockKeyInputResolver>();
        return if let Some(mock) = mock_input_resolver {
            Box::new(MockKeyInputResolver { key_results: mock.key_results.clone() })
        } else {
            Box::new(IoKeyInputResolver{})
        }
    }
    
    fn initial_prompt(&mut self) -> Result<Option<OpenInput>, ErrorWrapper> {
        self.ui.set_console_buffer("What do you want to open?. Arrow keys to choose. Repeat usage to choose current location.".to_string());
        self.re_render().unwrap();
        
        let mut input_resolver = self.get_input_resolver();
        
        let key = input_resolver.get_input_key()?;
        Ok(self.key_bindings.get_input(key).cloned())
    }
    
    fn find_container(&mut self, position: Position) -> Option<Container> {
        if let Some(map) = &mut self.level.map {
            if let Some(c) = map.containers.get(&position) {
                let item_count = c.get_top_level_count();
                if item_count > 0 {
                    log::info!("Found map container.");

                    // Automatically open any fixed container if it's the only item in this area container
                    // For example, a single Chest in the Floor container
                    let contains_single_container = item_count == 1 && c.get_contents()[0].is_fixed_container();
                    if contains_single_container && c.get_contents()[0].get_top_level_count() > 0 {
                        return Some(c.get_contents()[0].clone());
                    } else {
                        // Otherwise, show everything in this area container
                        return Some(c.clone());
                    }
                }
            }
        }
        None
    }
    
    pub(crate) async fn begin(&mut self) -> Result<(), ErrorWrapper> {
        let input_result = self.initial_prompt();
        if input_result.is_ok() {
            
            let input_maybe = input_result.unwrap();
            let side = map_open_input_to_side(input_maybe);
            if side.is_some() {
                let mut message = NOTHING_ERROR.to_string();
                if let Some(p) = self.level.find_adjacent_player_position(side) {
                    log::info!("Player opening at map position: {}, {}", &p.x, &p.y);
                    self.re_render()?;
    
                    let mut to_open : Option<Container> = self.find_container(p);
                    if let Some(c) = to_open {
                        self.ui.clear_console_buffer();
                        self.re_render()?;
                        log::info!("Player opening container of type {:?} and length: {}", c.container_type, c.get_total_count());
                        return self.open_container(p.clone(), &c).await;
                    } else {
                        return ErrorWrapper::internal_result(message)
                    }
                }
            }
        }
        return ErrorWrapper::internal_result(NOTHING_ERROR.to_string())
    }
    
    async fn open_container(&mut self, p: Position, c: &Container) -> Result<(), ErrorWrapper> {
        self.ui.set_console_buffer(UI_USAGE_HINT.to_string());

        log::info!("Player opening container: {} at position: {:?}", c.get_self_item().get_name(), p);
        
        let container = c.clone();
        let container_widget = ContainerWidget {
            columns: vec![
                Column {name : "NAME".to_string(), size: 30},
                Column {name : "STORAGE (Kg)".to_string(), size: 12}
            ],
            row_count: 1
        };

        let mut ui_layout = self.ui.ui_layout.clone().unwrap();
        let ui = &mut self.ui;

        // Add the container widget to the UI
        let stateful_widgets = ui.get_stateful_widgets_mut();
        stateful_widgets.push(StatefulWidgetType::Container(container_widget));

        let items = container.to_cloned_item_list();
        let mut item_list_selection =  ItemListSelection::new(items.clone(), 4);
        let commands : Vec<UsageCommand> = vec![
            UsageCommand::new('o', String::from("open") ),
            UsageCommand::new('t', String::from("take"))
        ];
        let usage_line = UsageLine::new(commands);
   
        // Run the re-render in a loop with the new widget set
        let level = self.level.clone();
        let container = container.clone();
        
        let frame_size = self.terminal_manager.terminal.get_frame().area();
        let ui_areas = ui_layout.get_or_build_areas(frame_size, LayoutType::StandardSplit);
        
        let mut event_handler = EventHandler::new();
        let event_thread = event_handler.spawn_thread();

        let mut widget_data = ContainerWidgetData {
            container: container.clone(),
            ui_areas: ui_areas.clone(),
            item_list_selection: item_list_selection,
            usage_line: usage_line.clone()
        };
        
        let mut running = true;
        while running {
            self.terminal_manager.terminal.draw(|frame| {
                ui.render(None, Some(&mut widget_data), frame);
            })?;

            let event = event_handler.receiver.recv().await;
            let stateful_widgets = ui.get_stateful_widgets_mut();
            let container_widget: &StatefulWidgetType = stateful_widgets.iter().find(|widget| widget.get_name() == "Container" ).unwrap();
            if let StatefulWidgetType::Container(container_widget) = container_widget {
                if let Some(e) = event {
                    match e {
                        Event::Termion(termion_event) => {
                            match termion_event { 
                                termion::event::Event::Key(key) => {
                                    match key {
                                        Key::Up => {
                                            widget_data.item_list_selection.move_up();
                                        },
                                        Key::Down => {
                                            widget_data.item_list_selection.move_down();
                                        },
                                        Key::Char('\n') => {
                                            widget_data.item_list_selection.toggle_select();
                                        },
                                        Key::Esc => {
                                            if (widget_data.item_list_selection.is_selecting()) {
                                                widget_data.item_list_selection.cancel_selection();
                                            } else {
                                                info!("Stopping Open Command Event Loop");
                                                running = false;
                                            }
                                        }
                                        _ => {}
                                    }
                                    
                                },
                                _ => {}
                            }
                            
                        },
                        Event::Tick => {
                            // TODO anything on tick?
                        }
                        _ => {}
                    }
                }

            } else {
                log::error!("Container widget not found.");
            }
        }
        
        log::info!("Open Command Event Loop finished");
        event_handler.receiver.close();
        event_thread.abort();
        Ok(())
    }
}