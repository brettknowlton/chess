pub mod pawn;
use pawn::Pawn;

pub mod knight;
use knight::Knight;

pub mod bishop;
use bishop::Bishop;

pub mod rook;
use rook::Rook;

pub mod queen;
use queen::Queen;

pub mod king;
use king::King;

use crate::{
    Board,
    app::board::{MoveNotation, position::Position},
};
use std::collections::HashMap;

use eframe::egui::{
    self, Align2, Color32, ColorImage, FontId, Rect, Response, TextureHandle, TextureOptions,
    Widget,
};
use egui_extras::image::load_image_bytes;

pub trait PieceTrait {
    ///returns a list of possible moves for this piece without checking for self-check (used for move generation and check detection)
    fn pseudo_targets(&self, board: &Board) -> Vec<MoveNotation>;

    fn get_color(&self) -> PieceColor;

    fn get_type(&self) -> PieceType;

    fn get_position(&self) -> Position;

    fn set_position(&mut self, new_position: Position);

    fn paint(&self, ui: &egui::Ui, rect: Rect, textures: &mut PieceTextures) {
        if let Some(handle) = textures.texture_for(ui.ctx(), self.get_color(), self.get_type()) {
            // Reference grid size for scaling calculations
            const REFERENCE_GRID_SIZE: f32 = 32.0;

            // Get the actual image dimensions
            let image_size = handle.size_vec2();

            // Calculate scale factor from reference grid to actual grid
            let grid_scale = rect.width() / REFERENCE_GRID_SIZE;

            // Scale the image proportionally
            let scaled_width = image_size.x * grid_scale;
            let scaled_height = image_size.y * grid_scale;

            // Center the scaled image within the grid square
            let image_rect =
                Rect::from_center_size(rect.center(), egui::vec2(scaled_width, scaled_height));

            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter()
                .image(handle.id(), image_rect, uv, Color32::WHITE);
        } else {
            // Fallback: draw a centered glyph
            let size = rect.height();
            let font_id = FontId::proportional(size * 0.8);
            let (glyph, color) = match self.get_color() {
                PieceColor::White => match self.get_type() {
                    PieceType::Pawn => ("♙", Color32::WHITE),
                    PieceType::Knight => ("♘", Color32::WHITE),
                    PieceType::Bishop => ("♗", Color32::WHITE),
                    PieceType::Rook => ("♖", Color32::WHITE),
                    PieceType::Queen => ("♕", Color32::WHITE),
                    PieceType::King => ("♔", Color32::WHITE),
                },
                PieceColor::Black => match self.get_type() {
                    PieceType::Pawn => ("♟", Color32::BLACK),
                    PieceType::Knight => ("♞", Color32::BLACK),
                    PieceType::Bishop => ("♝", Color32::BLACK),
                    PieceType::Rook => ("♜", Color32::BLACK),
                    PieceType::Queen => ("♛", Color32::BLACK),
                    PieceType::King => ("♚", Color32::BLACK),
                },
            };
            ui.painter()
                .text(rect.center(), Align2::CENTER_CENTER, glyph, font_id, color);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Piece {
    Pawn(Pawn),
    Knight(Knight),
    Bishop(Bishop),
    Rook(Rook),
    Queen(Queen),
    King(King),
}

impl Piece {
    pub fn new(piece_type: PieceType, color: PieceColor, position: Position) -> Self {
        match piece_type {
            PieceType::Pawn => Piece::Pawn(Pawn {
                color,
                position,
                has_moved: false,
            }),
            PieceType::Knight => Piece::Knight(Knight { color, position }),
            PieceType::Bishop => Piece::Bishop(Bishop { color, position }),
            PieceType::Rook => Piece::Rook(Rook {
                color,
                position,
                has_moved: false,
            }),
            PieceType::Queen => Piece::Queen(Queen { color, position }),
            PieceType::King => Piece::King(King {
                color,
                position,
                has_moved: false,
            }),
        }
    }
    pub fn to_string(&self) -> String {
        let icon = match (self.get_type(), self.get_color()) {
            (PieceType::Pawn, PieceColor::White) => "♙",
            (PieceType::Knight, PieceColor::White) => "♘",
            (PieceType::Bishop, PieceColor::White) => "♗",
            (PieceType::Rook, PieceColor::White) => "♖",
            (PieceType::Queen, PieceColor::White) => "♕",
            (PieceType::King, PieceColor::White) => "♔",

            (PieceType::Pawn, PieceColor::Black) => "♟",
            (PieceType::Knight, PieceColor::Black) => "♞",
            (PieceType::Bishop, PieceColor::Black) => "♝",
            (PieceType::Rook, PieceColor::Black) => "♜",
            (PieceType::Queen, PieceColor::Black) => "♛",
            (PieceType::King, PieceColor::Black) => "♚",
        };
        format!("{} at {}", icon, self.get_position())
    }
}

impl PieceTrait for Piece {
    fn get_color(&self) -> PieceColor {
        match self {
            Piece::Pawn(p) => p.color,
            Piece::Knight(k) => k.color,
            Piece::Bishop(b) => b.color,
            Piece::Rook(r) => r.color,
            Piece::Queen(q) => q.color,
            Piece::King(k) => k.color,
        }
    }

    fn get_type(&self) -> PieceType {
        match self {
            Piece::Pawn(_) => PieceType::Pawn,
            Piece::Knight(_) => PieceType::Knight,
            Piece::Bishop(_) => PieceType::Bishop,
            Piece::Rook(_) => PieceType::Rook,
            Piece::Queen(_) => PieceType::Queen,
            Piece::King(_) => PieceType::King,
        }
    }

    fn get_position(&self) -> Position {
        match self {
            Piece::Pawn(p) => p.position,
            Piece::Knight(k) => k.position,
            Piece::Bishop(b) => b.position,
            Piece::Rook(r) => r.position,
            Piece::Queen(q) => q.position,
            Piece::King(k) => k.position,
        }
    }

    fn set_position(&mut self, new_position: Position) {
        match self {
            Piece::Pawn(p) => p.position = new_position,
            Piece::Knight(k) => k.position = new_position,
            Piece::Bishop(b) => b.position = new_position,
            Piece::Rook(r) => r.position = new_position,
            Piece::Queen(q) => q.position = new_position,
            Piece::King(k) => k.position = new_position,
        }
    }

    fn pseudo_targets(&self, board: &Board) -> Vec<MoveNotation> {
        match self {
            Piece::Pawn(p) => p.get_soft_targets(board),
            Piece::Knight(n) => n.get_soft_targets(board),
            Piece::Bishop(b) => b.get_soft_targets(board),
            Piece::Rook(r) => r.get_soft_targets(board),
            Piece::Queen(q) => q.get_soft_targets(board),
            Piece::King(k) => k.get_soft_targets(board),
        }
    }
}

impl Widget for Piece {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        match self.get_color() {
            PieceColor::White => match self.get_type() {
                PieceType::Pawn => ui.add(
                    egui::Image::new(egui::include_image!("../../../../assets/pieces/WP.png"))
                        .corner_radius(5)
                        .texture_options(TextureOptions::NEAREST),
                ),
                PieceType::Knight => ui.label("♘"),
                PieceType::Bishop => ui.label("♗"),
                PieceType::Rook => ui.label("♖"),
                PieceType::Queen => ui.label("♕"),
                PieceType::King => ui.label("♔"),
            },
            PieceColor::Black => match self.get_type() {
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

#[derive(Clone)]
pub struct PieceTextures {
    decoded: HashMap<(PieceColor, PieceType), ColorImage>,
    handles: HashMap<(PieceColor, PieceType), TextureHandle>,
}

impl PieceTextures {
    pub fn load_from_disk() -> Self {
        let mut decoded = HashMap::new();
        let handles = HashMap::new();

        for &color in &[PieceColor::White, PieceColor::Black] {
            for &pt in &[
                PieceType::Pawn,
                PieceType::Knight,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Queen,
                PieceType::King,
            ] {
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
                let handle = ctx.load_texture(
                    name,
                    egui::ImageData::from(img.clone()),
                    TextureOptions::NEAREST,
                );
                self.handles.insert((color, piece_type), handle);
            } else {
                return None;
            }
        }
        self.handles.get(&(color, piece_type))
    }
}

pub fn piece_filename(color: PieceColor, piece_type: PieceType) -> String {
    let color_char = match color {
        PieceColor::White => 'W',
        PieceColor::Black => 'B',
    };
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
