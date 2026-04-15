mod board;
mod theme;

use std::{collections::HashMap, path::PathBuf};

pub use board::{Board, BoardWidget};
pub use theme::SkeletonTheme;

use eframe::{
    self,
    egui::{self, Align2, Color32, FontId},
};

use serde;

use crate::app::board::SelectedPiece;

static mut ID_COUNTER: usize = 0;

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
    board_state: Option<Board>,

    first_update: bool,

    theme_set: bool,
    color_values: HashMap<String, Color32>,
    #[serde(skip)]
    available_themes: Vec<SkeletonTheme>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),

            app_state: AppState::default(),

            board_state: None,

            first_update: true,

            theme_set: false,
            color_values: HashMap::new(),

            available_themes: Vec::new(),
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let new: Self;
        if let Some(_) = cc.storage {
            new = Default::default();
            // eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            new = Default::default();
        }

        let a = new.load_themes();
        a
    }

    pub fn next_id() -> usize {
        unsafe {
            ID_COUNTER += 1;
            ID_COUNTER
        }
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
                    .default_width(150.0)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Main Menu");
                            if ui.button("New Game").clicked() {
                                self.app_state = AppState::Playing;
                                self.board_state = Some(Board::new());
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
                    .default_width(150.0)
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

                egui::SidePanel::right("InfoPanel")
                    .default_width(150.0)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.heading("Info");
                            if let Some(board) = &self.board_state {
                                if let Some(SelectedPiece::Selected(idx, idy)) =
                                    &board.selected_piece
                                {
                                    let selected = board.pieces.get(&(*idx, *idy)).unwrap();
                                    ui.label(format!(
                                        "Selected Piece: {:?} {:?} at ({}, {})",
                                        selected.color,
                                        selected.piece_type,
                                        selected.position.0,
                                        selected.position.1
                                    ));
                                }
                            } else {
                                ui.label("No piece selected");
                            }
                        });
                    });

                egui::CentralPanel::default().show(ctx, |ui| {
                    //Load images/board.png as background
                    ui.horizontal(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(50.0);
                            let text;
                            let (a_color, b_color) = (Color32::WHITE, Color32::BLACK);
                            match self.board_state.as_ref().unwrap().turn {
                                board::GameTurn::WhiteTurn => {
                                    text = "White's Turn";
                                }
                                board::GameTurn::BlackTurn => {
                                    text = "Black's Turn";
                                }
                            };

                            ui.painter().text(
                                ui.min_rect().center_top() + egui::vec2(0.0, 10.0),
                                Align2::CENTER_TOP,
                                text,
                                FontId::proportional(20.0),
                                b_color,
                            );
                            ui.painter().text(
                                ui.min_rect().center_top() + egui::vec2(-3.0, 13.0),
                                Align2::CENTER_TOP,
                                text,
                                FontId::proportional(20.0),
                                a_color,
                            );
                        });
                    });
                    ui.centered_and_justified(|ui| {
                        if let Some(board_state) = &mut self.board_state {
                            ui.add(BoardWidget::new(board_state));
                        }
                    });
                });
            }

            AppState::Paused => {
                // Handle paused updates
            }
        }
    }
}
