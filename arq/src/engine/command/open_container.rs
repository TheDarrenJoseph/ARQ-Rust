use std::io;
use termion::event::Key;
use std::collections::HashMap;
use termion::input::TermRead;

use crate::view::framehandler::container_view;
use crate::engine::command::command::Command;
use crate::view::world_container_view::{WorldContainerViewFrameHandler, WorldContainerView};
use crate::view::framehandler::container_view::ContainerViewInputResult;
use crate::view::framehandler::container_view::ContainerViewInputResult::TAKE_ITEMS;
use crate::map::position::Position;
use crate::view::callback::Callback;
use crate::view::View;
use crate::map::Map;
use crate::engine::level::Level;

pub struct OpenContainerCommand<'a> {
    position : Option<Position>,
    level: &'a mut Level,
    callbacks : HashMap<String, Box<FnMut(ContainerViewInputResult)>>
}

impl Callback<'_, ContainerViewInputResult> for OpenContainerCommand<'_> {
    fn set_callback<'a>(&mut self, event_name: String, mut c: impl FnMut(ContainerViewInputResult) + 'static) {
        self.callbacks.insert(event_name, Box::new(c));
    }

    fn trigger_callback(&mut self, event_name: String, data: ContainerViewInputResult) {
        if self.callbacks.contains_key(&event_name) {
            let mut cb = self.callbacks.get_mut(&event_name).unwrap();
            cb(data);
        }
    }
}

impl Command for OpenContainerCommand<'_> {
    fn handles_key(&self, key: Key) -> bool {
        return match key {
            Key::Char('o') => {
                true
            }
            _ => {
                false
            }
        };
    }

    fn handle(&mut self, command_key: Key) -> Result<(), io::Error> {
        let p = &self.position.unwrap();
        if let Some(map) = &mut self.level.map {
            if let Some(room) = map.get_rooms().iter_mut().find(|r| r.area.contains_position(*p)) {
                if let Some(c) = room.containers.get(&p) {
                    log::info!("Player opening container.");
                    let mut inventory_container = c.clone();
                    let mut frame_container = c.clone();
                    let mut view_container = c.clone();
                    let mut container_view = container_view::build_container_view(inventory_container);
                    let result = ContainerViewInputResult::OPEN_CONTAINER_VIEW(container_view);
                    self.trigger_callback(String::from("open_container"), result);
                } else if let Some(door) = &room.doors.iter().find(|d| d.position == *p) {
                    log::info!("Player opening door.");
                    //self.ui.console_print("There's a door here.".to_string());
                    //self.re_render();
                } else {
                    //self.ui.console_print("There's nothing here to open.".to_string());
                    //self.re_render();
                }
            }


            let key = io::stdin().keys().next().unwrap().unwrap();
        }
        Ok(())
    }
}