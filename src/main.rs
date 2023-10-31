use eframe::egui;
use crate::amgui_app::AMGuiApp;

mod amgui_app;

//
// Main function
//
pub fn main() {
    // From: https://github.com/emilk/egui/blob/master/examples/hello_world/src/main.rs#L7
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),

        ..Default::default()
    };

    eframe::run_native(
        "Asset Migrator GUI",
        options,
        Box::new(|ctx| {
            Box::<AMGuiApp>::default()
        })
    ).unwrap();
}