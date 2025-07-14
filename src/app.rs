use std::io;
use std::error;
use hyprland::event_listener::Event as HyprlandEvent;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};


use ratatui::{backend::CrosstermBackend, Terminal};

use crate::tui::Tui;
use crate::event::{EventHandler, Event};
use crate::hyprlandwidget::HyprlandState;
use crate::batterywidget::BatteryState;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
pub enum Action {
    Render,
    UpdateHyprlandState(HyprlandEvent),
    Tick,
    None
}

pub struct App {
    pub running: bool,
    pub hyprland_state: HyprlandState,
    pub battery_state: BatteryState,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,

}

impl App {
    pub fn new() -> Self {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
        Self {
            running: true,
            hyprland_state: HyprlandState::new(),
            battery_state: BatteryState::new(),
            action_tx: action_tx.clone(),
            action_rx
        }
    }

    pub async fn run(&mut self) -> AppResult<()> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).expect("Failed to create backend");
        let events = EventHandler::new(50);
        let mut tui = Tui::new(terminal, events);
        tui.init().expect("Failed to inialize");
        while self.running {
            match tui.events.next().await? {
                Event::Tick => self.action_tx.send(Action::None)?,
                Event::Render => self.action_tx.send(Action::Render)?,
                Event::Mouse(_) => self.action_tx.send(Action::None)?,
                Event::Resize(_, _) => self.action_tx.send(Action::None)?,
                Event::UpdateHyprlandState(event) => self.action_tx.send(Action::UpdateHyprlandState(event))?,
            }

            while let Ok(action) = self.action_rx.try_recv() {
                let _ = self.update(action.clone()).await;
                if let Action::Render = action {
                    tui.draw(self)?;
                };
            }
        }
        Ok(())
    }

    async fn update(&mut self, action: Action) {
        match action {
            Action::UpdateHyprlandState(hyprland_event) => {
                match hyprland_event {
                    HyprlandEvent::WorkspaceChanged(workspace_event_data) => {
                        let mut i = 0;
                        for workspace in self.hyprland_state.workspaces.clone() {
                            if workspace_event_data.id == workspace.0 {
                                self.hyprland_state.activeworkspaceindex = i;
                                break;
                            }
                            i += 1;
                        }
                    },
                    HyprlandEvent::WorkspaceDeleted(workspace_event_data) => {
                        match self.hyprland_state.workspaces.binary_search_by_key(&workspace_event_data.id, |&(a,_)| a) {
                            Ok(pos) => {self.hyprland_state.workspaces.remove(pos);},
                            Err(_pos) => {}
                        } 
                    },
                    HyprlandEvent::WorkspaceAdded(workspace_event_data) => {
                        match self.hyprland_state.workspaces.binary_search_by_key(&workspace_event_data.id, |&(a,_)| a) {
                            Ok(_pos) => {}
                            Err(pos) => {self.hyprland_state.workspaces.insert(pos, (workspace_event_data.id, workspace_event_data.name.to_string()));}
                        } 
                    },
                    HyprlandEvent::ActiveWindowChanged(window_event_data) => {
                        self.hyprland_state.activewindow = match window_event_data {
                            Some(window_event) => window_event.title,
                            None => "".to_string(),
                        }
                    }
                    _ =>  {},
                }
            }
            Action::Tick => {
                self.battery_state.tick();
            }
            _ => {}
        }
    }
}
