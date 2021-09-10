use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use termion::{event::Key};

pub struct TerminalManager<B : tui::backend::Backend> {
    pub terminal : tui::Terminal<B>,
}

pub fn get_input_key() {
    loop {
        //log::info!("Loop {}", n);
        let input = io::stdin();
        let keys = input.keys();
        for key in keys {
            match key.unwrap() {
                Key::Char('q') => {
                    log::info!("Quitting...");
                    return;
                }
                Key::Char(c) => {
                    log::info!("Inputted: '{}'", c);
                }
                _ => ()
            }
        }
    }
}

pub fn init() -> Result<TerminalManager<TermionBackend<RawTerminal<io::Stdout>>>, io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let manager = TerminalManager { terminal };
    log::info!("Terminal initialised.");
    return Ok(manager);
}
