use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    widgets::{Paragraph, StatefulWidget, Tabs, Widget},
};

#[derive(Debug, Clone)]
pub struct PipewireState {
    pub volume: u8,
    pub muted: bool,
    pub default_sink_name: String,
    default_sink_id: u32,
}

impl PipewireState {
    pub fn new() -> Self {
        Self {
            volume: 100,
            muted: false,
            default_sink_name: "".to_string(),
            default_sink_id: 0,
        }
    }

    pub fn set_default_sink_id(&mut self, name: String, id: u32) {
        if name == self.default_sink_name {
            self.default_sink_id = id;
        }
    }

    pub fn update_volumes(&mut self, id: u32, floats: Vec<f32>) {
        if id == self.default_sink_id {
            self.volume = (floats[0].cbrt() * 100.0) as u8;
        }
    }
    pub fn update_muted(&mut self, id: u32, muted: bool) {
        if id == self.default_sink_id {
            self.muted = muted;
        }
    }
}

pub struct PipewireWidget {
    alignment: Alignment,
}

impl<'a> PipewireWidget {
    pub fn new() -> Self {
        Self {
            alignment: Alignment::Left,
        }
    }

    pub fn right_aligned(&mut self) {
        self.alignment = Alignment::Right;
    }
}

impl StatefulWidget for PipewireWidget {
    type State = PipewireState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PipewireState) {
        let icon = match state.muted {
            true => "󰝟",
            false => ["󰕿", "󰖀", "󰕾"][state.volume as usize / 34],
        };
        Paragraph::new(format!("{} {}", icon, state.volume))
            .alignment(self.alignment)
            .render(area, buf);
    }
}
