mod app;
mod editor;
mod render;
mod ui;

fn main() {
    let app = app::App::new();
    app.run();
}
