mod theme;
mod board_state;

use std::{collections::HashMap, path::PathBuf};

use crossbeam::channel::{Receiver, Sender};
pub use theme::SkeletonTheme;
pub use board_state::BoardState;

use eframe::{
    self,
    egui::{Color32, Grid, Pos2},
    *,
};

use serde;

#[derive(serde::Deserialize, serde::Serialize, Default)]
enum AppState {
    #[default]
    MainMenu,
    Playing,
    Paused,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyApp {
    // Example stuff:
    label: String,

    app_state: AppState,

    #[serde(skip)]
    board_state: Option<BoardState>,

    first_update: bool,

    theme_set: bool,
    color_values: HashMap<String, Color32>,
    #[serde(skip)]
    available_themes: Vec<SkeletonTheme>,

    #[serde(skip)]
    pub event_sender: Sender<UiEvent>,
    #[serde(skip)]
    pub event_receiver: Receiver<UiEvent>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (event_sender, event_receiver) = crossbeam::channel::unbounded();
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),

            app_state: AppState::default(),

            board_state: None,

            first_update: true,

            theme_set: false,
            color_values: HashMap::new(),
        
            available_themes: Vec::new(),

            event_sender,
            event_receiver,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (event_sender, event_receiver) = crossbeam::channel::unbounded();
        let mut new: Self;
        if let Some(_) = cc.storage {
            new = Default::default();
            // eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            new = Default::default();
        }

        new.event_sender = event_sender;
        new.event_receiver = event_receiver;
        let a = new.load_themes();
        a
    }

    pub fn load_themes(mut self) -> Self {
        //for item in ../../assets/*
        let theme_dir = PathBuf::from("assets/");
        if let Ok(entries) = std::fs::read_dir(theme_dir) {
            for entry in entries.flatten() {
                if let Some(path) = entry.path().to_str() {
                    if path.ends_with(".css") {
                        if let Ok(theme) = SkeletonTheme::from_css_file(path) {
                            println!("Loaded theme: {}", path);
                            self.available_themes.push(theme);
                        } else {
                            eprintln!("Failed to load theme from: {}", path);
                        }
                    }
                }
            }
        } else {
            eprintln!("Failed to read theme directory");
        }
        self
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
pub enum UiEvent {
    ClickedGate(usize, Pos2, bool), // id of the gate that was clicked, its position, and if it was a primary click
    ClickedWire(usize, Pos2, bool), // id of the wire that was clicked, its position, and if it was a primary click
    ClickedIO(usize, Pos2, bool), // id of clicked input or output, its position, and if it was a primary click
}

impl eframe::App for MyApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_update {
            self.first_update = false;
            egui_extras::install_image_loaders(ctx);
        }
        match self.app_state {
            AppState::MainMenu => {
                // Handle main menu updates
                egui::SidePanel::left("MMenuPanel")
                    .default_width(100.0)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Main Menu");
                            if ui.button("New Game").clicked() {
                                self.app_state = AppState::Playing;
                                self.board_state = Some(BoardState::new());
                            }
                            if ui.button("Quit").clicked() {
                                panic!("Quit button clicked"); // Replace with actual quit logic
                            }
                        });
                    });
            }

            AppState::Playing => {
                // Handle in-game updates
                //draw assets/images/chessboard.png as background
                egui::SidePanel::left("InGamePanel")
                    .default_width(100.0)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.heading("In Game");
                            if ui.button("Pause").clicked() {
                                self.app_state = AppState::Paused;
                            }
                            if ui.button("Main Menu").clicked() {
                                self.app_state = AppState::MainMenu;
                            }
                        });
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    //Load images/board.png as background
                    ui.horizontal(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(50.0);

                            ui.label("Chess Board");
                        });
                    });
                    ui.centered_and_justified(|ui| {
                        Grid::new("chess_grid")
                            .num_columns(8)
                            .spacing([0.0, 0.0])
                            .show(ui, |ui| {
                                for row in (0..8).rev() {
                                    for col in 0..8 {
                                        let is_light_square = (row + col) % 2 == 0;
                                        let square_color = if is_light_square {
                                            Color32::from_rgb(240, 217, 181) // Light square color
                                        } else {
                                            Color32::from_rgb(181, 136, 99) // Dark square color
                                        };
                                        let square_size = 60.0; // Size of each square

                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(square_size, square_size),
                                            egui::Sense::click(),
                                        );

                                        // Draw the square
                                        ui.painter().rect_filled(rect, 0.0, square_color);


                                        // Draw piece as an overlay using painter and cached textures (no layout impact)
                                        if let Some(board_state) = &mut self.board_state {
                                            for piece in &board_state.pieces {
                                                if piece.position == (col, row) {
                                                    piece.paint(ui, rect, &mut board_state.textures);
                                                }
                                            }
                                        }

                                        // Handle click events
                                        if response.clicked() {
                                            println!(
                                                "Square clicked: ({}, {})",
                                                row + 1,
                                                (b'a' + col as u8) as char
                                            );
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                });
            }

            AppState::Paused => {
                // Handle paused updates
            }
        }
    }
}
