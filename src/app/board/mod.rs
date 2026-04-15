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
        Self {
            pieces: Self::generate_from_notation(notation),
            piece_graveyard: Vec::new(),
            textures: PieceTextures::load_from_disk(),
            turn: GameTurn::default(),
            selected_piece: None,
        }
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

    ///takes in a notation string like "Pe2e4,Ng1f3,Bf1c4" and generates a list of pieces from it
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
    fn is_in_check(&mut self) -> (bool, bool) {
        //check if this color's king is being targeted by any opponent pieces
        //for good measure, find all targets for all pieces first
        let (mut white_in_check, mut black_in_check) = (false, false);
        for piece in self.pieces.values() {
            for t in &piece.clone().targets {
                if t.targets_enemy_king(&self) {
                    match piece.color {
                        PieceColor::White => {
                            println!("Black king is in check!");
                            black_in_check = true;
                        }
                        PieceColor::Black => {
                            println!("White king is in check!");
                            white_in_check = true;
                        }
                    }
                }
            }
        }

        (white_in_check, black_in_check)
    }

    /// Finds all targets for all pieces on the board and updates their target lists
    pub fn find_all_targets(&mut self) {
        //for each piece on the board, find its targets
        let mut pieces_clone = self.pieces.clone();
        for piece in pieces_clone.values_mut() {
            //run with depth zero so we get all targets even if they are checking as to not cause a recursion loop
            piece.find_targets(self.clone());
            self.pieces.get_mut(&piece.position).unwrap().targets = piece.targets.clone();
        }
    }

    ///takes in a move notation string like "Pe2e4" or "Nf3xd4" and returns a new Board with the move applied (clones self first as to not mutate self)
    pub fn simulate_move(&self, info: &MoveNotation) -> Self {
        let mut sim_board = self.clone();
        print!("Simulating move: ");
        sim_board.make_move(info.clone());
        println!("\nrefreshing sim_board");
        sim_board.find_all_targets();
        sim_board
    }

    ///make a move on the board, updating pieces, graveyard, turn, and selected_piece
    pub fn make_move(&mut self, info: MoveNotation) {
        //make a move on the board
        print!("{} ", info.to_string());

        let mut moved_piece = self
            .pieces
            .get(&(info.from_file, info.from_rank))
            .unwrap()
            .clone();
        moved_piece.position = (info.to_file, info.to_rank);
        moved_piece.targets = Vec::new();
        self.pieces.remove(&(info.from_file, info.from_rank));

        if !info.is_capture {
            //just move the piece
            self.pieces
                .insert((info.to_file, info.to_rank), moved_piece);
            //deselect the piece
            self.selected_piece = None;
            //change turn
            self.turn = match self.turn {
                GameTurn::WhiteTurn => GameTurn::BlackTurn,
                GameTurn::BlackTurn => GameTurn::WhiteTurn,
            };
            return;
        } else {
            //capture the piece at the destination square
            if !self.pieces.contains_key(&(info.to_file, info.to_rank)) {
                panic!(
                    "No piece to capture at ({}, {})",
                    info.to_file, info.to_rank
                );
            }
            let captured_piece = self.pieces.remove(&(info.to_file, info.to_rank)).unwrap();
            self.piece_graveyard.push(captured_piece);
            print!(": Captured piece: ({}{})", info.to_file, info.to_rank);

            self.pieces
                .insert((info.to_file, info.to_rank), moved_piece);
            //deselect the piece
            self.selected_piece = None;
            //change turn
            self.turn = match self.turn {
                GameTurn::WhiteTurn => GameTurn::BlackTurn,
                GameTurn::BlackTurn => GameTurn::WhiteTurn,
            };
        }
    }
}

/// A widget that renders the chess board with rank/file labels and handles square clicks.
pub struct BoardWidget<'a> {
    board: &'a mut Board,
    square_size: f32,
}

impl<'a> BoardWidget<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            square_size: 60.0,
        }
    }

    #[allow(dead_code)]
    pub fn square_size(mut self, size: f32) -> Self {
        self.square_size = size;
        self
    }

    fn click_on(&mut self, row: usize, col: usize) {
        if let Some(piece) = self.board.pieces.get(&(col, row)) {
            //there is a piece in this square
            println!("Clicked on piece: {:?} at ({}, {})", piece, col, row);
            if piece.color
                == match self.board.turn {
                    GameTurn::WhiteTurn => PieceColor::White,
                    GameTurn::BlackTurn => PieceColor::Black,
                }
            {
                //the piece we clicked on IS the color whose turn it is
                //so: select this piece

                //load this piece's targets because we just selected it
                let mut selected_piece = self.board.pieces.get(&(col, row)).unwrap().clone();

                self.board.find_all_targets();

                selected_piece.clean_self_checking_targets(&self.board);

                selected_piece.apply_check_statuses(&self.board);

                if let Some(piece) = self.board.pieces.get_mut(&(col, row)) {
                    piece.targets = selected_piece.targets.clone();
                }

                self.board.selected_piece = Some(SelectedPiece::Selected(col, row));

                println!("Selected piece at index ({}{})", col, row);
            } else {
                //we clicked on a piece of the other color
                //so: check if this square is a valid target for the selected piece

                //first, do we have a selected piece?
                if let Some(SelectedPiece::Selected(idx, idy)) = &self.board.selected_piece {
                    //we have a selected piece
                    //so: get the raw selected piece
                    let mut selected_piece = self.board.pieces.get(&(*idx, *idy)).unwrap().clone();
                    selected_piece.clean_self_checking_targets(&self.board);

                    //check if the clicked square is a valid target for the selected piece
                    if let Some(item) = selected_piece
                        .targets
                        .iter()
                        .find(|t| t.get_target_pos() == SquareNotation::from((col, row)))
                    {
                        //this is a valid target for this piece
                        //so: move the piece and capture if necessary
                        self.board.make_move(item.clone());
                        self.board.selected_piece = None;
                        return;
                    } else {
                        println!(
                            "Invalid move to ({}, {}) for selected piece, deselecting",
                            col, row
                        );
                        self.board.selected_piece = None;
                    }
                };
            }
        } else {
            //there is not a piece in this square
            //so: check if this square is a valid target for the selected piece
            //first, do we have a selected piece?
            if let Some(SelectedPiece::Selected(idx, idy)) = &self.board.selected_piece {
                //we have a selected piece
                //so: get the raw selected piece
                let mut selected_piece = self.board.pieces.get(&(*idx, *idy)).unwrap().clone();
                selected_piece.clean_self_checking_targets(&self.board);

                //check if the clicked square is a valid target for the selected piece
                if let Some(item) = selected_piece
                    .targets
                    .iter()
                    .find(|t| t.get_target_pos() == SquareNotation::from((col, row)))
                {
                    //this is a valid target for this piece
                    //so: move the piece and capture if necessary
                    self.board.make_move(item.clone());
                    self.board.selected_piece = None;
                    return;
                } else {
                    println!(
                        "Invalid move to ({}, {}) for selected piece, deselecting",
                        col, row
                    );
                    self.board.selected_piece = None;
                }
            };
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
                                &self.board.selected_piece
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
                            }
                            //if this square is a target for the selected piece, highlight it yellow
                            if let Some(SelectedPiece::Selected(idx, idy)) =
                                &self.board.selected_piece
                            {
                                let piece = self.board.pieces.get(&(*idx, *idy)).unwrap();

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

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(square_size, square_size),
                                Sense::click(),
                            );
                            total_rect = Some(total_rect.map(|r| r.union(rect)).unwrap_or(rect));

                            // Draw the square
                            ui.painter().rect_filled(rect, 0.0, square_color);

                            // Draw piece as an overlay using painter and cached textures (no layout impact)
                            for (id, piece) in &self.board.pieces {
                                if (id.0, id.1) == (col, row) {
                                    piece.paint(ui, rect, &mut self.board.textures);
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
