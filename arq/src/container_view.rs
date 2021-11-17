use crate::view::View;

pub struct ContainerView<'a, B : tui::backend::Backend> {
    pub container : Container,
    pub ui : &'a mut UI,
    pub terminal_manager : &'a mut TerminalManager<B>
}

impl <B : tui::backend::Backend> View for ContainerView<'_, B> {
    fn draw(&mut self) -> Result<(), Error> {
        let frame_handler = &mut self.frame_handler;
        let ui = &mut self.ui;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<bool, Error> {
        let key = io::stdin().keys().next().unwrap().unwrap();
        match key {
            Key::Char('q') => {
                self.terminal_manager.terminal.clear()?;
                return Ok(true)
            },
            Key::Char('\n') => {
            },
            Key::Char(c) => {
            },
            Key::Backspace => {
            },
            _ => {
            }
        }
        Ok(false)
    }
}