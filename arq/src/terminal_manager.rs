use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct TerminalManager<B : tui::backend::Backend> {
    pub terminal : tui::Terminal<B>,
}

pub fn init() -> Result<TerminalManager<TermionBackend<RawTerminal<io::Stdout>>>, io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let mut backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let manager = TerminalManager::<TermionBackend<RawTerminal<io::Stdout>>> { terminal };

    log::info!("Terminal initialised.");
    return Ok(manager);
}
