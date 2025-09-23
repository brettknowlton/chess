use std::collections::HashMap;
use eframe::egui::{self, Align2, Color32, ColorImage, FontId, Rect, Response, TextureHandle, TextureOptions, Widget};
use egui_extras::image::load_image_bytes;

pub struct BoardState {
    pub pieces: Vec<Piece>,
    pub textures: PieceTextures,
}

impl BoardState {
    pub fn new() -> Self {
        //read the string from the assets/boards/starter.txt file
        let path = "assets/boards/starter.txt";
        let notation = std::fs::read_to_string(path).unwrap().trim().to_string();
        println!("Loaded board with definition: {}", notation);
        Self {
            pieces: Piece::generate_from_notation(notation),
            textures: PieceTextures::load_from_disk(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub position: (usize, usize), // (file, rank)
}

impl Piece {
    pub fn generate_from_notation(notation: String) -> Vec<Piece> {
        //parse the notation string and generate pieces
        let pieces = notation
            .split(',')
            .map(|s| {
                println!("Adding piece from notation: {}", s);
                let c = &s[0..1];
                let color = match c {
                    "W" => PieceColor::White,
                    "B" => PieceColor::Black,
                    _ => panic!("Invalid color character: {}", c.to_string()),
                };

                let pt = &s[1..2];
                let piece_type: PieceType = match pt {
                    "P" => PieceType::Pawn,
                    "N" => PieceType::Knight,
                    "B" => PieceType::Bishop,
                    "R" => PieceType::Rook,
                    "Q" => PieceType::Queen,
                    "K" => PieceType::King,
                    _ => panic!("Invalid piece character"),
                };
                let file_char = s[2..3].chars().next().unwrap();
                let rank_char = s[3..4].chars().next().unwrap();

                let new_p = Piece {
                    piece_type,
                    color,
                    position: (
                        (file_char as u8 - b'a') as usize,
                        (rank_char.to_digit(10).unwrap() - 1) as usize,
                    ),
                };
                new_p
            })
            .collect();
        pieces
    }

    /// Paint this piece into the given rect using a texture if available, otherwise fall back to a Unicode glyph.
    pub fn paint(&self, ui: &egui::Ui, rect: Rect, textures: &mut PieceTextures) {
        if let Some(handle) = textures.texture_for(ui.ctx(), self.color, self.piece_type) {
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(handle.id(), rect, uv, Color32::WHITE);
        } else {
            // Fallback: draw a centered glyph
            let size = rect.height();
            let font_id = FontId::proportional(size * 0.8);
            let (glyph, color) = match self.color {
                PieceColor::White => match self.piece_type {
                    PieceType::Pawn => ("♙", Color32::WHITE),
                    PieceType::Knight => ("♘", Color32::WHITE),
                    PieceType::Bishop => ("♗", Color32::WHITE),
                    PieceType::Rook => ("♖", Color32::WHITE),
                    PieceType::Queen => ("♕", Color32::WHITE),
                    PieceType::King => ("♔", Color32::WHITE),
                },
                PieceColor::Black => match self.piece_type {
                    PieceType::Pawn => ("♟", Color32::BLACK),
                    PieceType::Knight => ("♞", Color32::BLACK),
                    PieceType::Bishop => ("♝", Color32::BLACK),
                    PieceType::Rook => ("♜", Color32::BLACK),
                    PieceType::Queen => ("♛", Color32::BLACK),
                    PieceType::King => ("♚", Color32::BLACK),
                },
            };
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, glyph, font_id, color);
        }
    }
}

impl Widget for Piece {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        match self.color {
            PieceColor::White => match self.piece_type {
                PieceType::Pawn => {
                    ui.add(
                        egui::Image::new(egui::include_image!("../../assets/pieces/WP.png"))
                            .corner_radius(5).texture_options(TextureOptions::NEAREST)
                    )
                },
                PieceType::Knight => ui.label("♘"),
                PieceType::Bishop => ui.label("♗"),
                PieceType::Rook => ui.label("♖"),
                PieceType::Queen => ui.label("♕"),
                PieceType::King => ui.label("♔"),
            },
            PieceColor::Black => match self.piece_type {
                PieceType::Pawn => ui.label("♟"),
                PieceType::Knight => ui.label("♞"),
                PieceType::Bishop => ui.label("♝"),
                PieceType::Rook => ui.label("♜"),
                PieceType::Queen => ui.label("♛"),
                PieceType::King => ui.label("♚"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceColor {
    White,
    Black,
}

pub struct PieceTextures {
    decoded: HashMap<(PieceColor, PieceType), ColorImage>,
    handles: HashMap<(PieceColor, PieceType), TextureHandle>,
}

impl PieceTextures {
    pub fn load_from_disk() -> Self {
        let mut decoded = HashMap::new();
        let handles = HashMap::new();

        for &color in &[PieceColor::White, PieceColor::Black] {
            for &pt in &[PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King] {
                let path = piece_filename(color, pt);
                match std::fs::read(&path) {
                    Ok(bytes) => match load_image_bytes(&bytes) {
                        Ok(img) => {
                            decoded.insert((color, pt), img);
                        }
                        Err(err) => eprintln!("Failed to decode image {}: {}", path, err),
                    },
                    Err(_) => {
                        // Missing image is fine; we'll fall back to glyphs
                        eprintln!("Missing piece image: {}", path);
                    }
                }
            }
        }

        Self { decoded, handles }
    }

    pub fn texture_for(
        &mut self,
        ctx: &egui::Context,
        color: PieceColor,
        piece_type: PieceType,
    ) -> Option<&TextureHandle> {
        if !self.handles.contains_key(&(color, piece_type)) {
            if let Some(img) = self.decoded.get(&(color, piece_type)) {
                let name = piece_filename(color, piece_type);
                let handle = ctx.load_texture(name, egui::ImageData::from(img.clone()), TextureOptions::NEAREST);
                self.handles.insert((color, piece_type), handle);
            } else {
                return None;
            }
        }
        self.handles.get(&(color, piece_type))
    }
}

fn piece_filename(color: PieceColor, piece_type: PieceType) -> String {
    let color_char = match color { PieceColor::White => 'W', PieceColor::Black => 'B' };
    let piece_char = match piece_type {
        PieceType::Pawn => 'P',
        PieceType::Knight => 'N',
        PieceType::Bishop => 'B',
        PieceType::Rook => 'R',
        PieceType::Queen => 'Q',
        PieceType::King => 'K',
    };
    format!("assets/pieces/{}{}.png", color_char, piece_char)
}
