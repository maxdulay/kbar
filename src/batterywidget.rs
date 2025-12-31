use std::fs::File;
use std::io::prelude::Read;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, Padding, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug, Clone)]
enum BatteryChargingState {
    Charging,
    Discharging,
}


#[derive(Debug, Clone)]
pub struct BatteryState {
    ticks: u8,
    pub capacity: usize,
    pub state: BatteryChargingState,
}

impl BatteryState {
    pub fn new() -> Self {
        let mut new_battery_state = Self {ticks:0, capacity:0, state:BatteryChargingState::Discharging };
        new_battery_state.update();
        new_battery_state
    }

    fn update(&mut self) {
        let mut charger = File::open("/sys/class/power_supply/ADP1/online").unwrap();
        let mut battery = File::open("/sys/class/power_supply/BAT0/capacity").unwrap();

        let mut buffer = [0; 3];

        let _ = charger.read(&mut buffer);
        self.state = if buffer[0] == ('1' as u8) {
            BatteryChargingState::Charging
        } else {
            BatteryChargingState::Discharging
        };
        let _ = battery.read(&mut buffer);
        self.capacity = if buffer[2] == ('0' as u8) {
            100
        } else if buffer[1] == 0 {
            buffer[0] - ('0' as u8)
        } else {
            (buffer[0] - ('0' as u8)) * 10 + (buffer[1] - '0' as u8)
        } as usize;
    }

    pub fn tick(&mut self) {
        self.ticks+=1 ;

        if self.ticks >= 100 {
            self.update();
            self.ticks = 0;
        }
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
            BatteryChargingState::Charging => {
                ["󰢟", "󰢜", "󰂆", "󰂇", "󰂈", "󰢝", "󰂉", "󰢞", "󰂊", "󰂋", "󰁹"][(state.capacity) / 10]
            }
            BatteryChargingState::Discharging => {
                ["󰂎", "󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"][(state.capacity) / 10]
            }
        };
        Paragraph::new(format!("{} {}%", icon, state.capacity))
            .alignment(self.alignment)
            .render(area, buf)
    }
}
