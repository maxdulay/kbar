use std::time::Duration;

use crossterm::event::{MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use hyprland::event_listener::{Event as HyprlandEvent};

use crate::app::AppResult;

/// Terminal events.

// #[derive(Clone, Copy, Debug)]
#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick.
    Render,
    Tick,
    /// Key press.
    // Key(KeyEvent),
    /// Mouse click/scroll.
    UpdateHyprlandState(HyprlandEvent),
    // HyprlandWorkspaceEvent(HyprlandEvent),
    // HyprlandWindowEvent(HyprlandEvent),
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
}

// TODO!: Implement IPC event handler here
impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut tick = tokio::time::interval(tick_rate);
            let mut reader = hyprland::event_listener::EventStream::new();
            loop {
                let tick_delay = tick.tick();
                let hyprland_event = reader.next().fuse();
                tokio::select! {
                _ = _sender.closed() => {
                  break;
                }
                _ = tick_delay => {
                  _sender.send(Event::Tick).unwrap();
                  _sender.send(Event::Render).unwrap();
                }
                Some(Ok(evt)) = hyprland_event => {
                        _sender.send(Event::UpdateHyprlandState(evt)).unwrap();
                      }
                  }
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> AppResult<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "This is an IO error",
            )))
    }
}
