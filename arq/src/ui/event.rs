use std::time::Duration;
use futures::{FutureExt, TryFutureExt};
use log::{debug, info};
use termion::AsyncReader;
use termion::input::TermRead;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::engine::command::open_command_new::OpenContainerEvent;

/*
    Based on example code from https://github.com/ratatui/templates
 */
#[derive(Debug)]
pub struct EventHandler {
    pub(crate) sender: mpsc::UnboundedSender<Event>,
    pub(crate) receiver: mpsc::UnboundedReceiver<Event>,
}


/*
    A thread that handles reading terminal events and emitting tick events on a regular schedule.
    Based on example code from https://github.com/ratatui/templates
 */
pub struct EventTask {
    sender: mpsc::UnboundedSender<Event>,
}

/*
    Based on example code from https://github.com/ratatui/templates
 */
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    // TODO migrate to crossterm events
    Termion(termion::event::Event),
    AppEvent(AppEvent)
}

pub enum AppEvent {
    OpenContainerEvent(OpenContainerEvent)
}

/*
    Based on example code from https://github.com/ratatui/templates
 */
impl EventHandler {
    pub fn new() -> EventHandler {
        let (sender, receiver) = mpsc::unbounded_channel();
        EventHandler { sender, receiver }
    }
    
    pub fn spawn_thread(&self) -> JoinHandle<()> {
        info!("Spawning event handler thread");
        let task = EventTask::new(self.sender.clone());
        tokio::spawn(async { task.run().await })
    }
}

impl EventTask {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }
    
    pub(crate) async fn run(self) {
        let tick_rate = Duration::from_secs_f64(0.5 as f64);
        let mut tick = tokio::time::interval(tick_rate);
        let mut events = termion::async_stdin().events();
        
        loop {
            while let Some(e) = events.next() {
                match e {
                    Ok(e) => {
                        if self.sender.is_closed() {
                            break;
                        }
                        //debug!("Sending Termion event: {:?}", e);
                        self.send(Event::Termion(e));
                    },
                    Err(e) => {
                        println!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
            // wait for the next tick before trying to read events
            //debug!("TICK");
            tick.tick().await;
            self.send(Event::Tick);
            //debug!("TOCK");
        }
    }
    
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}
