use hyprland::event_listener::Event as HyprlandEvent;
use std::error;
use std::io;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::batterywidget::BatteryState;
use crate::event::{Event, EventHandler};
use crate::hyprlandwidget::HyprlandState;
use crate::pipemon::PipeWireEvent;
use crate::pipewirewidget::PipewireState;
use crate::networkwidget::NetworkState;
use crate::tui::Tui;

use crate::network::nl80211_stream::Event as NetEvent;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
pub enum Action {
    Render,
    UpdateHyprlandState(HyprlandEvent),
    UpdatePipeWireState(PipeWireEvent),
    UpdateNetworkState(NetEvent),
    Tick,
    None,
}

pub struct App {
    pub running: bool,
    pub hyprland_state: HyprlandState,
    pub battery_state: BatteryState,
    pub pipwire_state: PipewireState,
    pub network_state: NetworkState,
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
            pipwire_state: PipewireState::new(),
            network_state: NetworkState::new(),
            action_tx: action_tx.clone(),
            action_rx,
        }
    }

    pub async fn run(&mut self) -> AppResult<()> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).expect("Failed to create backend");
        let events = EventHandler::new(100);
        let mut tui = Tui::new(terminal, events);
        tui.init().expect("Failed to inialize");
        while self.running {
            match tui.events.next().await? {
                Event::Tick => self.action_tx.send(Action::Tick)?,
                Event::Render => self.action_tx.send(Action::Render)?,
                Event::Mouse(_) => self.action_tx.send(Action::None)?,
                Event::Resize(_, _) => self.action_tx.send(Action::None)?,
                Event::UpdateHyprlandState(event) => {
                    self.action_tx.send(Action::UpdateHyprlandState(event))?
                }
                Event::UpdatePipeWireState(event) => self.action_tx.send(Action::UpdatePipeWireState(event))?,
                Event::UpdateNetworkState(event) => self.action_tx.send(Action::UpdateNetworkState(event))?
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
            Action::UpdateHyprlandState(hyprland_event) => match hyprland_event {
                HyprlandEvent::WorkspaceChanged(workspace_event_data) => {
                    let mut i = 0;
                    for workspace in self.hyprland_state.workspaces.clone() {
                        if workspace_event_data.id == workspace.0 {
                            self.hyprland_state.activeworkspaceindex = i;
                            break;
                        }
                        i += 1;
                    }
                }
                HyprlandEvent::WorkspaceDeleted(workspace_event_data) => {
                    match self
                        .hyprland_state
                        .workspaces
                        .binary_search_by_key(&workspace_event_data.id, |&(a, _)| a)
                    {
                        Ok(pos) => {
                            self.hyprland_state.workspaces.remove(pos);
                        }
                        Err(_pos) => {}
                    }
                }
                HyprlandEvent::WorkspaceAdded(workspace_event_data) => {
                    match self
                        .hyprland_state
                        .workspaces
                        .binary_search_by_key(&workspace_event_data.id, |&(a, _)| a)
                    {
                        Ok(_pos) => {}
                        Err(pos) => {
                            self.hyprland_state.workspaces.insert(
                                pos,
                                (
                                    workspace_event_data.id,
                                    workspace_event_data.name.to_string(),
                                ),
                            );
                        }
                    }
                }
                HyprlandEvent::ActiveWindowChanged(window_event_data) => {
                    self.hyprland_state.activewindow = match window_event_data {
                        Some(window_event) => window_event.title,
                        None => "".to_string(),
                    }
                }
                HyprlandEvent::ActiveMonitorChanged(monitor_event_data) => {
                    if let Some(workspace_type) = monitor_event_data.workspace_name {
                        let mut i = 0;
                        let name = workspace_type.to_string();
                        for workspace in self.hyprland_state.workspaces.clone() {
                            if name == workspace.1 {
                                self.hyprland_state.activeworkspaceindex = i;
                                break;
                            }
                            i += 1;
                        }
                    }
                }
                _ => {}
            },
            Action::UpdatePipeWireState(pipewire_event) => match pipewire_event {
                PipeWireEvent::UpdateVolumes(id, items) => {
                    self.pipwire_state.update_volumes(id, items)
                }
                PipeWireEvent::UpdateMuted(id, muted) => self.pipwire_state.update_muted(id, muted),
                PipeWireEvent::SetDefaultSinkName(name) => {
                    self.pipwire_state.default_sink_name = name;
                }
                PipeWireEvent::UpdateNodeId(id, name) => {
                    self.pipwire_state.set_default_sink_id(name, id)
                }
            },

            Action::UpdateNetworkState(network_event) => match network_event {
                NetEvent::Connect(_, ifindex) => {
                    if let Some(ifindex) = ifindex {
                        self.network_state.connected(ifindex);
                    }
                },
                NetEvent::Disconnect => {
                    self.network_state.disconnected();
                },
                _ => ()
            },
            Action::Tick => {
                self.battery_state.tick();
                self.network_state.tick();
            }
            _ => {}
        }
    }
}
