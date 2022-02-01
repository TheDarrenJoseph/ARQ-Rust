pub trait Callback<'b, COM: 'b>{
    fn set_callback<'a>(&mut self, c : Box<impl FnMut(COM) + 'b>);
    fn trigger_callback(&mut self, data: COM);
}
