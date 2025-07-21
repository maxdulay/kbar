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
use crate::pipewirewidget::PipewireWidget;

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout= Layout::horizontal([Constraint::Ratio(1,3), Constraint::Ratio(1,3), Constraint::Ratio(1,3)]).split(frame.area());
    let left = Layout::horizontal([Constraint::Min(20), Constraint::Percentage(100)]).split(layout[0]);
    let right = Layout::horizontal([Constraint::Percentage(100), Constraint:: Min(7), Constraint::Min(9)]).split(layout[2]);
    let clock = Paragraph::new(format!(
        "{}",
        chrono::offset::Local::now().format("%a %b %d %H:%M")
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
    let mut pwstate = app.pipwire_state.clone();
    let mut pipewire = PipewireWidget::new();
    pipewire.right_aligned();
    frame.render_stateful_widget(pipewire, right[1], &mut pwstate);
    let mut battery = BatteryWidget::new();
    battery.right_aligned();
    let mut batstate = app.battery_state.clone();
    frame.render_stateful_widget(battery, right[2], &mut batstate);
}
