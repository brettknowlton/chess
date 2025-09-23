pub use eframe::egui;

pub mod app;
pub use app::*;



fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_min_inner_size([600.0, 600.0]),
        ..Default::default()
        
    };
    eframe::run_native(
        "Chess",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}