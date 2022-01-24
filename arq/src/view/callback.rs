use std::io;

pub trait Callback<'b, COM: 'b>{
    fn set_callback<'a>(&mut self, event_name: String, c : impl FnMut(COM) + 'static);
    fn trigger_callback(&mut self, event_name: String, data: COM);
}