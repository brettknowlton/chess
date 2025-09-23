use std::collections::HashMap;

use eframe::egui::{
    self, Align2, Color32, ColorImage, FontId, Rect, Response, TextureHandle, TextureOptions,
    Widget,
};
use egui_extras::image::load_image_bytes;

use crate::{
    MyApp,
    app::board::{self, piece},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub position: (usize, usize), // (file, rank)
    pub targets: Vec<String>,     //possible target squares
}

impl Piece {
    /// Paint this piece into the given rect using a texture if available, otherwise fall back to a Unicode glyph.
    pub fn paint(&self, ui: &egui::Ui, rect: Rect, textures: &mut PieceTextures) {
        if let Some(handle) = textures.texture_for(ui.ctx(), self.color, self.piece_type) {
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
            ui.painter()
                .text(rect.center(), Align2::CENTER_CENTER, glyph, font_id, color);
        }
    }

    pub fn find_targets(self, board: board::Board) -> Vec<String> {
        //find possible target squares for this piece
        let direction: isize = match self.color {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };
        let mut targets = vec![];
        let (file, rank) = self.position;

        match self.piece_type {
            PieceType::Pawn => {
                //pawns move differently based on color
                let forward_rank = (rank as isize + direction) as usize;

                if forward_rank < 8 {
                    if board.pieces.get(&(file, forward_rank)).is_none() {
                        //only add forward move if square is empty
                        let tg = (file, forward_rank);
                        targets.push(self.target_as_notation(tg, false, board.test_check(self.color, tg)));
                    }
                    //initial double move
                    if (self.color == PieceColor::White && rank == 1)
                        || (self.color == PieceColor::Black && rank == 6)
                    {
                        let double_forward_rank = (rank as isize + 2 * direction) as usize;
                        if board.pieces.get(&(file, double_forward_rank)).is_none()
                            && board.pieces.get(&(file, forward_rank)).is_none()
                        {
                            //only add forward move if square is empty
                            let tg = (file, double_forward_rank);
                            targets.push(self.target_as_notation(tg, false, false));
                        }
                    }
                }

                //captures
                if file > 0 {
                    if let Some(seen_piece) = board.pieces.get(&(file - 1, forward_rank)) {
                        if seen_piece.color != self.color {
                            let tg = (file - 1, forward_rank);
                            targets.push(self.target_as_notation(tg, true, false));
                        }
                    }
                }
                if file < 8 {
                    if let Some(seen_piece) = board.pieces.get(&(file + 1, forward_rank)) {
                        if seen_piece.color != self.color {
                            let tg = (file + 1, forward_rank);
                            targets.push(self.target_as_notation(tg, true, false));
                        }
                    }
                }
            }
            PieceType::Knight => {
                let (file, rank) = self.position;
                let knight_moves = [
                    (2, 1),
                    (1, 2),
                    (-1, 2),
                    (-2, 1),
                    (-2, -1),
                    (-1, -2),
                    (1, -2),
                    (2, -1),
                ];

                for km in knight_moves {
                    let (df, dr) = (km.0, km.1);
                    let target_file = file as isize + df;
                    let target_rank = rank as isize + dr;
                    if target_file >= 0 && target_file < 8 && target_rank >= 0 && target_rank < 8 {
                        if let Some(seen_piece) = board
                            .pieces
                            .get(&(target_file as usize, target_rank as usize))
                        {
                            if seen_piece.color != self.color {
                                let tg = (target_file as usize, target_rank as usize);
                                targets.push(self.target_as_notation(tg, true, false));
                            }
                        } else {
                            let tg = (target_file as usize, target_rank as usize);
                            targets.push(self.target_as_notation(tg, false, false));
                        }
                    }
                }
            }
            PieceType::Rook => {
                //horizontal and vertical lines until blocked
                let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                for dir in directions.iter() {
                    let (df, dr) = *dir;
                    let mut step = 1;
                    loop {
                        let target_file = file as isize + df * step;
                        let target_rank = rank as isize + dr * step;
                        if target_file < 0
                            || target_file >= 8
                            || target_rank < 0
                            || target_rank >= 8
                        {
                            break;
                        }
                        let target_pos = (target_file as usize, target_rank as usize);
                        if let Some(seen_piece) = board.pieces.get(&target_pos) {
                            if seen_piece.color != self.color {
                                let tg = (target_file as usize, target_rank as usize);
                                targets.push(self.target_as_notation(tg, true, false));
                            }
                            break; //blocked by any piece
                        } else {
                            let tg = (target_file as usize, target_rank as usize);
                            targets.push(self.target_as_notation(tg, false, false));
                        }
                        step += 1;
                    }
                }
            }
            PieceType::Bishop => {
                //diagonal lines until blocked
                let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
                for dir in directions.iter() {
                    let (df, dr) = *dir;
                    let mut step = 1;
                    loop {
                        let target_file = file as isize + df * step;
                        let target_rank = rank as isize + dr * step;
                        if target_file < 0
                            || target_file >= 8
                            || target_rank < 0
                            || target_rank >= 8
                        {
                            break;
                        }
                        let target_pos = (target_file as usize, target_rank as usize);
                        if let Some(seen_piece) = board.pieces.get(&target_pos) {
                            if seen_piece.color != self.color {
                                let tg = (target_file as usize, target_rank as usize);
                                targets.push(self.target_as_notation(tg, true, false));
                            }
                            break; //blocked by any piece
                        } else {
                            let tg = (target_file as usize, target_rank as usize);
                            targets.push(self.target_as_notation(tg, false, false));
                        }
                        step += 1;
                    }
                }
            }
            PieceType::Queen => {
                //horizontal, vertical, and diagonal lines until blocked
                let directions = [
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ];
                for dir in directions.iter() {
                    let (df, dr) = *dir;
                    let mut step = 1;
                    loop {
                        let target_file = file as isize + df * step;
                        let target_rank = rank as isize + dr * step;
                        if target_file < 0
                            || target_file >= 8
                            || target_rank < 0
                            || target_rank >= 8
                        {
                            break;
                        }
                        let target_pos = (target_file as usize, target_rank as usize);
                        if let Some(seen_piece) = board.pieces.get(&target_pos) {
                            if seen_piece.color != self.color {
                                let tg = (target_file as usize, target_rank as usize);
                                targets.push(self.target_as_notation(tg, true, false));
                            }
                            break; //blocked by any piece
                        } else {
                            let tg = (target_file as usize, target_rank as usize);
                            targets.push(self.target_as_notation(tg, false, false));
                        }
                        step += 1;
                    }
                }
            }
            PieceType::King => {
                let king_moves = [
                    (1, 0),
                    (1, 1),
                    (0, 1),
                    (-1, 1),
                    (-1, 0),
                    (-1, -1),
                    (0, -1),
                    (1, -1),
                ];
                for km in king_moves {
                    let (df, dr) = km;
                    let target_file = file as isize + df;
                    let target_rank = rank as isize + dr;
                    if target_file >= 0 && target_file < 8 && target_rank >= 0 && target_rank < 8 {
                        if let Some(seen_piece) = board
                            .pieces
                            .get(&(target_file as usize, target_rank as usize))
                        {
                            if seen_piece.color != self.color {
                                let tg = (target_file as usize, target_rank as usize);
                                targets.push(self.target_as_notation(tg, true, false));
                            }
                        } else {
                            let tg = (target_file as usize, target_rank as usize);
                            targets.push(self.target_as_notation(tg, false, false));
                        }
                    }
                }
            }
            _ => {
                //other pieces not implemented yet
            }
        }
        let clean_t: Vec<String> = targets
            .iter_mut()
            .filter_map(|t| {
                if board.move_causes_self_check(t.to_string()) {
                    None
                } else {
                    Some(t.to_string())
                }
            })
            .collect::<Vec<String>>();
        clean_t
    }

    fn target_as_notation(
        &self,
        target: (usize, usize),
        is_capture: bool,
        is_check: bool,
    ) -> String {
        let color = match self.color {
            PieceColor::White => "W",
            PieceColor::Black => "B",
        };
        let piece = match self.piece_type {
            PieceType::Pawn => "P",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K",
        };

        let from_rank = (b'1' + self.position.1 as u8) as char;
        let from_file = (b'a' + self.position.0 as u8) as char;

        let opt_capture = if is_capture { "x" } else { "" };
        let opt_check = if is_check { "+" } else { "" };

        let file_char = (b'a' + target.0 as u8) as char;
        let rank_char = (b'1' + target.1 as u8) as char;
        format!(
            "{}{}{}{}{}{}{}{}",
            color, piece, from_rank, from_file, opt_capture, file_char, rank_char, opt_check
        )
    }

    pub fn parse_move_notation(
        notation: String,
    ) -> (
        PieceColor,
        PieceType,
        (usize, usize),
        (usize, usize),
        bool,
        bool,
    ) {
        let chars: Vec<char> = notation.chars().collect();
        if chars.len() < 4 {
            panic!("Invalid move notation: {}", notation);
        }
        let color = match chars[0] {
            'W' => PieceColor::White,
            'B' => PieceColor::Black,
            _ => panic!("Invalid color character: {}", chars[0]),
        };
        let piece = match chars[1] {
            'P' => PieceType::Pawn,
            'N' => PieceType::Knight,
            'B' => PieceType::Bishop,
            'R' => PieceType::Rook,
            'Q' => PieceType::Queen,
            'K' => PieceType::King,
            _ => panic!("Invalid piece character: {}", chars[1]),
        };
        let from_rank = (chars[2] as u8 - b'1') as usize;
        let from_file = (chars[3] as u8 - b'a') as usize;

        let is_check = chars.contains(&'+');
        let is_capture = chars.contains(&'x');
        let mut to_file = (chars[5] as u8 - b'a') as usize;
        let mut to_rank = (chars[6] as u8 - b'1') as usize;

        if is_capture {
            if chars.len() < 6 {
                panic!("Invalid move notation for capture: {}", notation);
            }
        } else {
            to_file = (chars[5] as u8 - b'a') as usize;
            to_rank = (chars[6] as u8 - b'1') as usize;
        }
        return (
            color,
            piece,
            (from_file, from_rank),
            (to_file, to_rank),
            is_capture,
            is_check,
        );
    }

    pub fn test_check(self, color: PieceColor, tg: (usize, usize)) {
        //this function will test if some move X will result in a check for the opponent

    }
}

impl Widget for Piece {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        match self.color {
            PieceColor::White => match self.piece_type {
                PieceType::Pawn => ui.add(
                    egui::Image::new(egui::include_image!("../../../assets/pieces/WP.png"))
                        .corner_radius(5)
                        .texture_options(TextureOptions::NEAREST),
                ),
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
