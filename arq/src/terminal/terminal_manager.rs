use std::io;

use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::{TermionBackend, TestBackend};
use tui::Terminal;

pub struct TerminalManager<B : tui::backend::Backend> {
    pub terminal : tui::Terminal<B>,
}

impl <B : tui::backend::Backend>  TerminalManager<B> {
    pub fn clear_screen(&mut self) -> Result<(), io::Error> {
        self.terminal.clear()
    }
}

pub fn init() -> Result<TerminalManager<TermionBackend<RawTerminal<io::Stdout>>>, io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let manager = TerminalManager::<TermionBackend<RawTerminal<io::Stdout>>> { terminal };

    log::info!("Terminal initialised.");
    return Ok(manager);
}

pub fn init_test() -> Result<TerminalManager<TestBackend>, io::Error> {
    let backend = TestBackend::new(10,10);
    let terminal = Terminal::new(backend)?;
    let manager = TerminalManager::<TestBackend> { terminal };

    log::info!("Terminal initialised.");
    return Ok(manager);
}
