/*
    This trait is used to provide UI event hookup (i.e Open container, Drop item, etc)
    This specific usage of callbacks intends to allow both a call and return value (handled in trigger_callback, using the same DATA type)
 */
pub trait Callback<'b, DATA: 'b>{
    fn set_callback<'a>(&mut self, c : Box<impl FnMut(DATA) -> Option<DATA> + 'b>);
    fn trigger_callback(&mut self, data: DATA);
    fn handle_callback_result(&mut self, data: Option<DATA>);
}

/*
    This is a basic receiver of the callback, that then potentially returns the data with an optional result back
    Currently this is not widely implemented as there's additional data not wrapped in DATA objects in certain scenarios.
 */
pub trait CallbackHandler<DATA> {
    fn handle_callback(&mut self, data: DATA) -> Option<DATA>;
}