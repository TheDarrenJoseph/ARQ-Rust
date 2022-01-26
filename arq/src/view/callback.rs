pub trait Callback<'b, COM: 'b>{
    fn set_callback<'a>(&mut self, event_name: String, c : Box<impl FnMut(COM) + 'b>);
    fn trigger_callback(&mut self, event_name: String, data: COM);
}
