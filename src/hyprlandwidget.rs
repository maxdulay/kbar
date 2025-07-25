use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{StatefulWidget, Tabs, Widget},
};

use hyprland::{
    data::{Clients, Workspaces},
    shared::HyprData,
};

#[derive(Debug, Clone)]
pub struct HyprlandState {
    pub workspaces: Vec<(i32, String)>,
    // activeworkspace: String,
    pub activeworkspaceindex: usize,
    pub activewindow: String,
}

impl HyprlandState {
    pub fn new() -> Self {
        let hyprworkspaces = Workspaces::get().unwrap();
        let mut workspaces = hyprworkspaces
            .iter()
            .map(|workspace| (workspace.id, workspace.name.clone()))
            .collect::<Vec<(i32, String)>>();
        workspaces.sort_by(|a, b| a.0.cmp(&b.0));
        let clients = Clients::get().unwrap();
        let activewindow = clients.iter().find(|&x| x.focus_history_id == 0);

        Self {
            workspaces,
            activeworkspaceindex: 0,
            activewindow: match activewindow {
                Some(client) => client.title.to_string(),
                None => "".to_string(),
            },
        }
    }
}

pub struct HyprlandWorkSpaceWidget {}

impl<'a> HyprlandWorkSpaceWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl StatefulWidget for HyprlandWorkSpaceWidget {
    type State = HyprlandState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut HyprlandState) {
        let highlight_style = (Color::Black, Color::Blue);
        let workspace_num = state.workspaces.len()-1;
        Tabs::new(
            state
                .workspaces
                .iter()
                .map(|workspace| format!(" {} ", workspace.1))
                .collect::<Vec<String>>(),
        )
        .padding(" ", " ")
        .highlight_style(highlight_style)
        .select(if state.activeworkspaceindex > workspace_num {
            workspace_num
        } else {
            state.activeworkspaceindex
        })
        .render(area, buf);
    }
}
