#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod document;
mod editor;
mod render;
mod ui;

fn main() {
    let app = app::App::new();
    app.run();
}
