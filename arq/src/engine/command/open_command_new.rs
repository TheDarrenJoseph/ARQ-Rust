use std::io;
use crate::engine::command::command::Command;
use crate::engine::command::open_command::OpenCommand;
use crate::engine::level::Level;
use crate::engine::message::channels::MessageChannels;
use crate::error::errors::ErrorWrapper;
use crate::input::{IoKeyInputResolver, KeyInputResolver, MockKeyInputResolver};
use crate::map::objects::container::Container;
use crate::map::position::Position;
use crate::terminal::terminal_manager::TerminalManager;
use crate::ui::bindings::input_bindings::KeyBindings;
use crate::ui::bindings::open_bindings::{map_open_input_to_side, OpenInput, OpenKeyBindings};
use crate::ui::ui::{get_input_key, UI};
use crate::view::framehandler::container;
use crate::view::InputResult;
use crate::view::model::usage_line::{UsageCommand, UsageLine};
use crate::view::world_container_view::{WorldContainerView, WorldContainerViewFrameHandlers};

pub struct OpenCommandNew<'a, B: 'static + ratatui::backend::Backend> {
    pub level: &'a mut Level,
    pub ui: &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>,
    pub input_resolver: Box<dyn KeyInputResolver>,
    pub key_bindings: OpenKeyBindings
}

const UI_USAGE_HINT: &str = "Up/Down - Move\nEnter/q - Toggle/clear selection\nEsc - Exit";
const NOTHING_ERROR : &str = "There's nothing here to open.";

impl <B: ratatui::backend::Backend> OpenCommandNew<'_, B> {

    fn re_render(&mut self) -> Result<(), io::Error>  {
        let ui = &mut self.ui;
        let level = self.level.clone();
        self.terminal_manager.terminal.draw(|frame| {
            ui.render(Some(level), frame);
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
    
    pub(crate) fn begin(&mut self) -> Result<(), ErrorWrapper> {
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
                        return self.open_container(p.clone(), &c);
                    } else {
                        return ErrorWrapper::internal_result(message)
                    }
                }
            }
        }
        return ErrorWrapper::internal_result(NOTHING_ERROR.to_string())
    }

    fn open_container(&mut self, p: Position, c: &Container) -> Result<(), ErrorWrapper> {
        self.ui.set_console_buffer(UI_USAGE_HINT.to_string());

        log::info!("Player opening container: {} at position: {:?}", c.get_self_item().get_name(), p);
        let subview_container = c.clone();
        let view_container = c.clone();

        let commands : Vec<UsageCommand> = vec![
            UsageCommand::new('o', String::from("open") ),
            UsageCommand::new('t', String::from("take"))
        ];
        let usage_line = UsageLine::new(commands);

    
        let level = &mut self.level;

        //let message_channels = MessageChannels::new();

        // let result_receiver = Some(message_channels.request_channel.receiver);
        // let mut result_sender = Some(message_channels.response_channel.sender);
        
        // TODO replace view with a widget
        
        return Ok(())
    }
}