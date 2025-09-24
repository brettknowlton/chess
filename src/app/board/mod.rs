use core::panic;
use std::collections::HashMap;

use eframe::egui::{self, Align2, Color32, FontId, Grid, Rect, Sense, Widget};

pub mod piece;
pub use piece::{Piece, PieceColor, PieceTextures};

pub mod notations;
pub use notations::MoveNotation;

use crate::app::board::{notations::SquareNotation, piece::PieceType};

#[derive(Clone)]
pub struct Board {
    pub pieces: HashMap<(usize, usize), Piece>,
    pub piece_graveyard: Vec<Piece>,
    pub textures: PieceTextures,
    pub turn: GameTurn,
    pub selected_piece: Option<SelectedPiece>,
}

impl Board {
    pub fn new() -> Self {
        //read the string from the assets/boards/starter.txt file
        let path = "assets/boards/starter.txt";
        let notation = std::fs::read_to_string(path).unwrap();
        let notation = notation.trim();
        println!("Loaded board with definition: {}", notation);
        let mut a = Self {
            pieces: Self::generate_from_notation(notation),
            piece_graveyard: Vec::new(),
            textures: PieceTextures::load_from_disk(),
            turn: GameTurn::default(),
            selected_piece: None,
        };
        a = a.fill_all_targets();
        a
    }

    pub fn file_to_char(file: usize) -> char {
        match file {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => panic!("Invalid file index"),
        }
    }

    pub fn rank_to_char(rank: usize) -> char {
        match rank {
            0 => '1',
            1 => '2',
            2 => '3',
            3 => '4',
            4 => '5',
            5 => '6',
            6 => '7',
            7 => '8',
            _ => panic!("Invalid rank index"),
        }
    }

    pub fn fill_all_targets(&mut self) -> Self {
        let board_copy = self.clone();

        for piece in self.pieces.values_mut() {
            piece.targets = Piece::find_targets(piece.clone(), board_copy.clone());
        }
        board_copy
    }

    pub fn generate_from_notation(notation: &str) -> HashMap<(usize, usize), Piece> {
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
                    targets: Vec::new(),
                };

                (new_p.position, new_p)
            })
            .collect();
        pieces
    }

    /// Returns (white_in_check, black_in_check), true if that color's king is in check for this board
    fn is_in_check(self) -> (bool, bool) {
        //check if this color's king is being targeted by any opponent pieces
        let (mut w_in_check, mut b_in_check) = (false, false);
        for ((_file, _rank), piece) in self.pieces.iter() {
            
            let b_not_pos;

            if let Some(b_king_pos) = self
                .pieces
                .values()
                .find(|p| p.piece_type == PieceType::King && p.color == PieceColor::Black)
            {
                b_not_pos = SquareNotation::from(b_king_pos.position);
            } else {
                continue;
            };

            let w_not_pos;
            if let Some(w_king_pos) = self
                .pieces
                .values()
                .find(|p| p.piece_type == PieceType::King && p.color == PieceColor::White)
            {
                w_not_pos = SquareNotation::from(w_king_pos.position);
            } else {
                continue;
            };

            if piece.color == PieceColor::White {
                for t in piece.targets.clone() {
                    if t.targets_square(&b_not_pos) {
                        b_in_check = true;
                    }
                }
            } else {
                for t in piece.targets.clone() {
                    if t.targets_square(&w_not_pos) {
                        w_in_check = true;
                    }
                }
            }
        }
        (w_in_check, b_in_check)
    }
    
    ///takes in a move notation string like "Pe2e4" or "Nf3xd4" and returns a new Board with the move applied (clones self first as to not mutate self)
    pub fn simulate_move(&self, info: &MoveNotation) -> Self {
        let mut sim_board = self.clone();

        println!("Moving piece from ({}, {}) to ({}, {})", info.from_file, info.from_rank, info.to_file, info.to_rank);

        let mut moved_piece = sim_board.pieces.get(&(info.from_file, info.from_rank)).unwrap().clone();
        moved_piece.position = (info.to_file, info.to_rank);
        moved_piece.targets = Vec::new();
        sim_board.pieces.remove(&(info.from_file, info.from_rank));

        if !info.is_capture {
            //just move the piece
            sim_board.pieces.insert((info.to_file, info.to_rank), moved_piece);
            //deselect the piece
            sim_board.selected_piece = None;
            
            //change turn
            sim_board.turn = match sim_board.turn {
                GameTurn::WhiteTurn => GameTurn::BlackTurn,
                GameTurn::BlackTurn => GameTurn::WhiteTurn,
            };

            return sim_board;
        } else{
            //capture the piece at the destination square
            if !sim_board.pieces.contains_key(&(info.to_file, info.to_rank)) {
                panic!("No piece to capture at ({}, {})", info.to_file, info.to_rank);
            }
            let captured_piece = sim_board.pieces.remove(&(info.to_file, info.to_rank)).unwrap();
            sim_board.piece_graveyard.push(captured_piece);
            println!("Captured piece at ({}, {})", info.to_file, info.to_rank);
        }

        sim_board.pieces.insert((info.to_file, info.to_rank), moved_piece);
        //deselect the piece
        sim_board.selected_piece = None;
        //change turn
        sim_board.turn = match sim_board.turn {
            GameTurn::WhiteTurn => GameTurn::BlackTurn,
            GameTurn::BlackTurn => GameTurn::WhiteTurn,
        };

        //recalculate all targets
        sim_board.pieces = sim_board.fill_all_targets().pieces;
        sim_board
    }

    pub fn make_move(&mut self, info: MoveNotation) {
        //make a move on the board
        println!("Moving piece from ({}, {}) to ({}, {})", info.from_file, info.from_rank, info.to_file, info.to_rank);

        let mut moved_piece = self.pieces.get(&(info.from_file, info.from_rank)).unwrap().clone();
        moved_piece.position = (info.to_file, info.to_rank);
        moved_piece.targets = Vec::new();
        self.pieces.remove(&(info.from_file, info.from_rank));

        if !info.is_capture {
            //just move the piece
            self.pieces.insert((info.to_file, info.to_rank), moved_piece);
            //deselect the piece
            self.selected_piece = None;
            //change turn
            self.turn = match self.turn {
                GameTurn::WhiteTurn => GameTurn::BlackTurn,
                GameTurn::BlackTurn => GameTurn::WhiteTurn,
            };
            return;
        } else{
            //capture the piece at the destination square
            if !self.pieces.contains_key(&(info.to_file, info.to_rank)) {
                panic!("No piece to capture at ({}, {})", info.to_file, info.to_rank);
            }
            let captured_piece = self.pieces.remove(&(info.to_file, info.to_rank)).unwrap();
            self.piece_graveyard.push(captured_piece);
            println!("Captured piece at ({}, {})", info.to_file, info.to_rank);
        }

        self.pieces.insert((info.to_file, info.to_rank), moved_piece);
        //deselect the piece
        self.selected_piece = None;
        //change turn
        self.turn = match self.turn {
            GameTurn::WhiteTurn => GameTurn::BlackTurn,
            GameTurn::BlackTurn => GameTurn::WhiteTurn,
        };

        //recalculate all targets
        self.pieces = self.fill_all_targets().pieces;
    }


}

/// A widget that renders the chess board with rank/file labels and handles square clicks.
pub struct BoardWidget<'a> {
    state: &'a mut Board,
    square_size: f32,
}

impl<'a> BoardWidget<'a> {
    pub fn new(state: &'a mut Board) -> Self {
        Self {
            state,
            square_size: 60.0,
        }
    }

    #[allow(dead_code)]
    pub fn square_size(mut self, size: f32) -> Self {
        self.square_size = size;
        self
    }

    fn click_on(&mut self, row: usize, col: usize) {
        if let Some(piece) = self.state.pieces.get(&(col, row)) {
            //there is a piece in this square
            println!("Clicked on piece: {:?} at ({}, {})", piece, col, row);
            if piece.color
                == match self.state.turn {
                    GameTurn::WhiteTurn => PieceColor::White,
                    GameTurn::BlackTurn => PieceColor::Black,
                }
            {
                //the piece we clicked on IS the color whose turn it is
                //so: select this piece
                self.state.selected_piece = Some(SelectedPiece::Selected(col, row));
                //load this piece's targets because we just selected it
                self.state.pieces.get_mut(&(col, row)).unwrap().targets =
                    Piece::find_targets(piece.clone(), self.state.clone());

                println!("Selected piece at index ({}{})", col, row);
            } else {
                //we clicked on a piece of the other color
                //so: check if this square is a valid target for the selected piece
                if let Some(SelectedPiece::Selected(idx, idy)) = &self.state.selected_piece {
                    //we have a selected piece
                    //so:
                    let selected_piece = self.state.pieces.get(&(*idx, *idy)).unwrap().clone();

                    //check if the clicked square is a valid target for the selected piece
                    selected_piece.targets.iter().for_each(|t| {
                        if t.get_target_pos() == SquareNotation::from((col, row)) {
                            //this is a valid target for this piece
                            //so: capture the piece and send it to the graveyard
                            self.state.make_move(t.clone());
                        } else {
                            println!(
                                "Invalid move to ({}, {}) for selected piece, deselecting",
                                col, row
                            );
                        }
                        self.state.selected_piece = None;
                    });
                };
            }
        } else {
            //there is not a piece in this square
            //so: check if this square is a valid target for the selected piece
            if let Some(SelectedPiece::Selected(idx, idy)) = &self.state.selected_piece {
                let selected_piece = self.state.pieces.get(&(*idx, *idy)).unwrap().clone();

                selected_piece.targets.iter().for_each(|t| {
                        if t.get_target_pos() == SquareNotation::from((col, row)) {
                            //this is a valid target for this piece
                            //so: capture the piece and send it to the graveyard
                            self.state.make_move(t.clone());
                        } else {
                            println!(
                                "Invalid move to ({}, {}) for selected piece, deselecting",
                                col, row
                            );
                        }
                        self.state.selected_piece = None;
                    });
            } else {
                println!("No piece at ({}, {}) and no piece is selected.", col, row);
            }
        }
    }
}

impl<'a> Widget for BoardWidget<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let square_size = self.square_size;
        let mut total_rect: Option<Rect> = None;

        ui.centered_and_justified(|ui| {
            Grid::new("chess_grid")
                .num_columns(9)
                .spacing([0.0, 0.0])
                .show(ui, |ui| {
                    // Board rows (top 8 rows)
                    for row in (0..8).rev() {
                        // Left rank label cell (non-clickable)
                        let (rrect, _rresp) = ui.allocate_exact_size(
                            egui::vec2(square_size, square_size),
                            Sense::hover(),
                        );

                        total_rect = Some(total_rect.map(|r| r.union(rrect)).unwrap_or(rrect));
                        ui.painter().rect_filled(rrect, 0.0, Color32::from_gray(60));

                        let rank_label = (row + 1).to_string();
                        let font_id = FontId::proportional(square_size * 0.4);
                        ui.painter().text(
                            rrect.center(),
                            Align2::CENTER_CENTER,
                            rank_label,
                            font_id,
                            Color32::WHITE,
                        );

                        // 8 board squares with pieces and click handling
                        for col in 0..8 {
                            let square_notation = SquareNotation::from((col, row));

                            let is_light_square = (row + col) % 2 == 0;
                            let mut square_color = if is_light_square {
                                Color32::from_rgb(0, 0, 0)
                            } else {
                                Color32::from_rgb(155, 173, 183)
                            };

                            //if this square is the one with the selected piece, highlight it green
                            if let Some(SelectedPiece::Selected(idx, idy)) =
                                &self.state.selected_piece
                            {
                                if (idx, idy) == (&col, &row) {
                                    //highlight square green
                                    //blend the colors
                                    let r = (square_color.r() as u16 + 0x008800u16) / 2;
                                    let g = (square_color.g() as u16 + 0x00FF00u16) / 2;
                                    let b = (square_color.b() as u16 + 0x008800u16) / 2;
                                    let blended_color =
                                        Color32::from_rgb(r as u8, g as u8, b as u8);
                                    //use the blended color
                                    square_color = blended_color;
                                }

                                //if this square is a target for the selected piece, highlight it yellow
                                if let Some(SelectedPiece::Selected(idx, idy)) =
                                    &self.state.selected_piece
                                {
                                    let piece = self.state.pieces.get(&(*idx, *idy)).unwrap();

                                    for t in &piece.targets {
                                        if t.get_target_pos() == square_notation {
                                            //highlight square yellow
                                            //blend the colors
                                            let r = (square_color.r() as u16 + 0x00FF00u16) / 2;
                                            let g = (square_color.g() as u16 + 0x00FF00u16) / 2;
                                            let b = (square_color.b() as u16 + 0x000000u16) / 2;
                                            let blended_color =
                                                Color32::from_rgb(r as u8, g as u8, b as u8);
                                            //use the blended color
                                            square_color = blended_color;
                                        }
                                    }
                                }
                            }

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(square_size, square_size),
                                Sense::click(),
                            );
                            total_rect = Some(total_rect.map(|r| r.union(rect)).unwrap_or(rect));

                            // Draw the square
                            ui.painter().rect_filled(rect, 0.0, square_color);

                            // Draw piece as an overlay using painter and cached textures (no layout impact)
                            for (id, piece) in &self.state.pieces {
                                if (id.0, id.1) == (col, row) {
                                    piece.paint(ui, rect, &mut self.state.textures);
                                }
                            }

                            // Handle click events (only for real board squares)
                            if response.clicked() {
                                self.click_on(row, col);
                            }
                        }
                        ui.end_row();
                    }

                    // Bottom file labels row
                    // Left-bottom corner label cell (non-clickable, blank)
                    let (c_rect, _c_resp) = ui
                        .allocate_exact_size(egui::vec2(square_size, square_size), Sense::hover());
                    total_rect = Some(total_rect.map(|r| r.union(c_rect)).unwrap_or(c_rect));
                    ui.painter()
                        .rect_filled(c_rect, 0.0, Color32::from_gray(60));

                    for col in 0..8 {
                        let (frect, _fresp) = ui.allocate_exact_size(
                            egui::vec2(square_size, square_size),
                            Sense::hover(),
                        );
                        total_rect = Some(total_rect.map(|r| r.union(frect)).unwrap_or(frect));
                        ui.painter().rect_filled(frect, 0.0, Color32::from_gray(60));
                        let file_label = ((b'a' + col as u8) as char).to_string();
                        let font_id = FontId::proportional(square_size * 0.4);
                        ui.painter().text(
                            frect.center(),
                            Align2::CENTER_CENTER,
                            file_label,
                            font_id,
                            Color32::WHITE,
                        );
                    }
                    ui.end_row();
                });
        });

        // Return an overall response that covers the board area (hover-only)
        if let Some(tr) = total_rect {
            let id = ui.make_persistent_id("chess_board");
            ui.interact(tr, id, Sense::hover())
        } else {
            ui.allocate_exact_size(egui::vec2(0.0, 0.0), Sense::hover())
                .1
        }
    }
}

#[derive(Default, Clone)]
pub enum GameTurn {
    #[default]
    WhiteTurn,
    BlackTurn,
}

#[derive(Clone)]
pub enum SelectedPiece {
    None,
    Selected(usize, usize), //index of the piece in the pieces vector
}
