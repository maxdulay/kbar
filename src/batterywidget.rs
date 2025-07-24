use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, Padding, Paragraph, StatefulWidget, Widget},
};

use battery::{Manager, State as BatteryChargingState};

#[derive(Debug, Clone)]
pub struct BatteryState {
    ticks: u8,
    pub charge: usize,
    pub state: BatteryChargingState,
}

impl BatteryState {
    pub fn new() -> Self {
        let battery = match Manager::new().unwrap().batteries().unwrap().next() {
            Some(battery) => battery.unwrap(),
            None => todo!(),
        };
        Self {
            ticks: 0,
            charge: (battery.state_of_charge().value * 100.0) as usize,
            state: battery.state(),
        }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks >= 100 {
            self.update();
            self.ticks = 0;
        }
    }

    pub fn update(&mut self) {
        if let Ok(battery) = Manager::new().unwrap().batteries().unwrap().next().unwrap() {
            self.charge = (battery.state_of_charge().value * 100.0) as usize;
            self.state = battery.state();
        };
    }
}

pub struct BatteryWidget {
    alignment: Alignment,
}

impl<'a> BatteryWidget {
    pub fn new() -> Self {
        Self {
            alignment: Alignment::Left,
        }
    }
    pub fn right_aligned(&mut self) {
        self.alignment = Alignment::Right;
    }
    pub fn center_aligned(&mut self) {
        self.alignment = Alignment::Center;
    }
}

impl StatefulWidget for BatteryWidget {
    type State = BatteryState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut BatteryState) {
        let icon = match state.state {
            BatteryChargingState::Unknown => " ",
            BatteryChargingState::Charging => {
                ["󰢟", "󰢜", "󰂆", "󰂇", "󰂈", "󰢝", "󰂉", "󰢞", "󰂊", "󰂋", "󰁹"]
                    [(state.charge) / 10]
            }
            BatteryChargingState::Discharging => {
                ["󰂎", "󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"]
                    [(state.charge) / 10]
            }
            BatteryChargingState::Empty => "󰂎",
            BatteryChargingState::Full => "󰁹",
            _ => " "
        };
        Paragraph::new(format!("{} {}%", icon, state.charge))
            .alignment(self.alignment)
            .render(area, buf)
    }
}
