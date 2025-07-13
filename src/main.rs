use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::{App, AppResult},
};

pub mod app;
pub mod tui;
pub mod ui;
pub mod event;
pub mod hyprlandwidget;
pub mod batterywidget;

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut app = App::new();
    let result = app.run().await;
    Ok(())
}
