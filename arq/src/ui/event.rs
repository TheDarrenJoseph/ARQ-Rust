use std::time::Duration;
use log::{debug, error, info};
use termion::input::TermRead;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use crate::engine::command::open_command_new::OpenContainerEventType;

/*
    Based on example code from https://github.com/ratatui/templates
 */
#[derive(Debug)]
pub struct TerminalEventHandler {
    pub(crate) sender: mpsc::UnboundedSender<Event>,
    pub(crate) receiver: mpsc::UnboundedReceiver<Event>,
}


/*
    A thread that handles reading terminal events and emitting tick events on a regular schedule.
    Based on example code from https://github.com/ratatui/templates
 */
pub struct EventTask {
    cancellation_token: CancellationToken,
    sender: mpsc::UnboundedSender<Event>,
}

/*
    Based on example code from https://github.com/ratatui/templates
 */
#[derive(Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    // TODO migrate to crossterm events
    Termion(termion::event::Event),
    AppEvent(AppEventType)
}

#[derive(Debug)]
pub enum AppEventType {
    OpenContainerEvent(OpenContainerEventType)
}

/*
    Based on example code from https://github.com/ratatui/templates
 */
impl TerminalEventHandler {
    pub fn new() -> TerminalEventHandler {
        let (sender, receiver) = mpsc::unbounded_channel();
        TerminalEventHandler { sender, receiver }
    }
    
    pub fn spawn_thread(&self) -> TerminalEventThreadData {
        let cancellation_token = CancellationToken::new();
        
        info!("Spawning event handler thread");
        let task = EventTask::new(
            cancellation_token.clone(),
            self.sender.clone()
        );
        let join_handle = tokio::spawn(async { task.run().await });
        
        TerminalEventThreadData {
            cancellation_token,
            join_handle
        }
    }
}

pub struct TerminalEventThreadData {
    pub cancellation_token: CancellationToken,
    pub join_handle: JoinHandle<()>
}

impl EventTask {
    pub fn new(cancellation_token: CancellationToken, sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { 
            cancellation_token,
            sender 
        }
    }
    
    pub(crate) async fn run(self) {
        let tick_rate = Duration::from_secs_f64(0.1f64);
        let mut tick = tokio::time::interval(tick_rate);
        let mut events = termion::async_stdin().events();
        
        loop {
            if self.cancellation_token.is_cancelled() {
                info!("Event Task cancelled");
                return;         
            }
            
            if let Some(e) = events.next() {
                match e {
                    Ok(e) => {
                        if self.sender.is_closed() {
                            break;
                        }
                        debug!("Sending Termion event: {:?}", e);
                        self.send(Event::Termion(e));
                    },
                    Err(e) => {
                        error!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
            // wait for the next tick before trying to read events
            tick.tick().await;
            debug!("TICK");
            // self.send(Event::Tick);
            // debug!("TOCK");
        }
    }
    
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}
