use chrono;
use ratatui::{
    Frame,
    layout::{Rect, Constraint, Layout, Position, Flex},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListState, Padding, Paragraph},

};

use crate::app::App;
use crate::hyprlandwidget::{HyprlandWorkSpaceWidget};
use crate::batterywidget::BatteryWidget;

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout= Layout::horizontal([Constraint::Ratio(1,3), Constraint::Ratio(1,3), Constraint::Ratio(1,3)]).split(frame.area());
    let left = Layout::horizontal([Constraint::Min(17), Constraint::Percentage(100)]).split(layout[0]);
    let clock = Paragraph::new(format!(
        "{}",
        chrono::offset::Local::now().format("%H:%M %m/%d/%Y")
    ))
    .block(Block::new().padding(Padding {
        left: 2,
        right: 2,
        top: 0,
        bottom: 0,
    }));
    frame.render_widget(clock, left[0]);
    let workspaces = HyprlandWorkSpaceWidget::new();
    let mut hyprstate = app.hyprland_state.clone();
    frame.render_stateful_widget(workspaces, left[1], &mut hyprstate);
    let activewindow = Paragraph::new(app.hyprland_state.activewindow.clone()).centered();
    frame.render_widget(activewindow, layout[1]);
    let mut battery = BatteryWidget::new();
    battery.right_aligned();
    let mut batstate = app.battery_state.clone();
    frame.render_stateful_widget(battery, layout[2], &mut batstate);
}
