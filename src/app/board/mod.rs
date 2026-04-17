use core::panic;
use std::collections::HashMap;

use eframe::egui::{self, Align2, Color32, FontId, Grid, Rect, Sense, Widget};

pub mod position;
pub use piece::{Piece, PieceColor, PieceTextures};
pub use position::Position;
pub mod piece;

pub mod notations;
pub use notations::MoveNotation;

use crate::app::board::piece::{PieceTrait, PieceType};

#[derive(Clone)]
pub struct Board {
    pub pieces: HashMap<Position, Piece>,
    pub piece_graveyard: Vec<Piece>,
    pub textures: PieceTextures,
    pub turn: GameTurn,
    pub selected_piece_location: Option<Position>,
    pub selected_targets: Vec<MoveNotation>,
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
            selected_piece_location: None,
            selected_targets: Vec::new(),
        }
    }

    pub fn is_check_mate(&self) -> Option<PieceColor> {
        //check if the current player is in check, and if they have any legal moves to get out of check
        let current_color = match self.turn {
            GameTurn::WhiteTurn => PieceColor::White,
            GameTurn::BlackTurn => PieceColor::Black,
        };
        let current_color_in_check = self.is_in_check(current_color);
        if !current_color_in_check {
            return None;
        }

        //if we are in check, do we have any legal moves to get out of check?
        for piece in self.pieces.values() {
            if piece.get_color()
                == match self.turn {
                    GameTurn::WhiteTurn => PieceColor::White,
                    GameTurn::BlackTurn => PieceColor::Black,
                }
            {
                if !self.legal_targets_for(piece).is_empty() {
                    return None;
                }
            }
        }

        match self.turn {
            GameTurn::WhiteTurn => Some(PieceColor::White),
            GameTurn::BlackTurn => Some(PieceColor::Black),
        }
    }

    pub fn to_notation(&self) -> String {
        //generate a notation string like "Pe2e4,Ng1f3,Bf1c4" from the current board state
        self.pieces
            .values()
            .map(|piece| {
                let color_char = match piece.get_color() {
                    PieceColor::White => 'W',
                    PieceColor::Black => 'B',
                };
                let piece_char = match piece.get_type() {
                    PieceType::Pawn => 'P',
                    PieceType::Knight => 'N',
                    PieceType::Bishop => 'B',
                    PieceType::Rook => 'R',
                    PieceType::Queen => 'Q',
                    PieceType::King => 'K',
                };

                let file_char = piece.get_position().file;
                let rank_char = piece.get_position().rank;

                format!("{}{}{}{}", color_char, piece_char, file_char, rank_char)
            })
            .collect::<Vec<String>>()
            .join(",")
    }

    ///takes in a notation string like "Pe2e4,Ng1f3,Bf1c4" and generates a list of pieces from it
    pub fn generate_from_notation(notation: &str) -> HashMap<Position, Piece> {
        //parse the notation string and generate pieces
        let pieces: HashMap<Position, Piece> = notation
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

                let new_p = Piece::new(
                    piece_type,
                    color,
                    Position::new(file_char, (rank_char.to_digit(10).unwrap()) as u8),
                );

                (new_p.get_position(), new_p)
            })
            .collect();
        pieces
    }

    /// Returns true if the specified color's king is in check for this board
    fn is_in_check(&self, color: PieceColor) -> bool {
        let king_position = self
            .pieces
            .values()
            .find(|piece| piece.get_color() == color && piece.get_type() == PieceType::King)
            .map(|piece| piece.get_position());

        let Some(king_position) = king_position else {
            panic!("No king found for color {:?}", color);
        };

        self.pieces
            .values()
            .filter(|piece| piece.get_color() != color)
            .any(|piece| {
                self.pseudo_targets_for(piece)
                    .iter()
                    .any(|movement| movement.targets_square(&king_position))
            })
    }

    fn pseudo_targets_for(&self, piece: &Piece) -> Vec<MoveNotation> {
        piece.pseudo_targets(self)
    }

    fn move_leaves_own_king_in_check(&self, piece_color: PieceColor, info: &MoveNotation) -> bool {
        let simulated_board = self.simulate_move(info);
        simulated_board.is_in_check(piece_color)
    }

    fn move_puts_enemy_in_check(&self, piece_color: PieceColor, info: &MoveNotation) -> bool {
        let simulated_board = self.simulate_move(info);
        let enemy_color = match piece_color {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        simulated_board.is_in_check(enemy_color)
    }

    pub fn legal_targets_for(&self, piece: &Piece) -> Vec<MoveNotation> {
        let mut items = self.pseudo_targets_for(piece);
        let cleaned_items = items
            .iter_mut()
            .filter_map(|target| {
                if self.move_leaves_own_king_in_check(piece.get_color(), target) {
                    target.is_self_check = true;
                    return None;
                }

                target.is_check = self.move_puts_enemy_in_check(piece.get_color(), target);
                Some(target.clone())
            })
            .collect::<Vec<MoveNotation>>();
        cleaned_items
    }

    pub fn select_piece(&mut self, piece: &Piece) {
        self.selected_targets = self.legal_targets_for(piece);
        self.selected_piece_location = Some(piece.get_position());
    }

    pub fn clear_selection(&mut self) {
        self.selected_piece_location = None;
        self.selected_targets.clear();
    }

    pub fn selected_piece_location(&self) -> Option<Position> {
        self.selected_piece_location
    }

    pub fn selected_legal_targets(&self) -> Vec<MoveNotation> {
        self.selected_targets.clone()
    }

    pub fn selected_move_to(&self, position: Position) -> Option<MoveNotation> {
        self.selected_legal_targets()
            .iter()
            .find(|target| target.get_target_pos() == position)
            .cloned()
    }

    ///takes in a move notation string like "Pe2e4" or "Nf3xd4" and returns a new Board with the move applied (clones self first as to not mutate self)
    pub fn simulate_move(&self, info: &MoveNotation) -> Self {
        let mut sim_board = self.clone();
        sim_board.apply_move(info, false);
        sim_board
    }

    ///make a move on the calling board, updating pieces, graveyard, turn, and selected_piece
    pub fn make_move(&mut self, info: &MoveNotation) {
        self.apply_move(info, true);
    }

    fn apply_move(&mut self, info: &MoveNotation, log_move: bool) {
        //make a move on the board
        if log_move {
            print!("{} ", info.to_string());
        }

        let mut moved_piece = self.pieces.get(&info.moving_from_position).unwrap().clone();
        moved_piece.set_position(info.moving_to_position);
        self.pieces.remove(&info.moving_from_position);

        if !info.is_capture {
            //just move the piece
            self.pieces.insert(info.moving_to_position, moved_piece);
            //deselect the piece
            self.clear_selection();
        } else {
            //capture the piece at the destination square
            if !self.pieces.contains_key(&info.moving_to_position) {
                panic!("No piece to capture at ({})", info.moving_to_position);
            }
            let captured_piece = self.pieces.remove(&info.moving_to_position).unwrap();
            self.piece_graveyard.push(captured_piece);
            if log_move {
                print!(": Captured piece: ({})", info.moving_to_position,);
            }

            self.pieces.insert(info.moving_to_position, moved_piece);
            //deselect the piece
            self.clear_selection();
        }
        //change turn
        self.turn = match self.turn {
            GameTurn::WhiteTurn => GameTurn::BlackTurn,
            GameTurn::BlackTurn => GameTurn::WhiteTurn,
        };
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

    fn click_on(&mut self, pos: Position) {
        //first off- are we clicking on a piece or an empty square?
        if let Some(piece) = self.board.clone().pieces.get(&pos) {
            //there is a piece in this square
            println!("Clicked on piece:  at ({})", pos);

            //is this piece the color of the player whose turn it is?
            if piece.get_color()
                == match self.board.turn {
                    GameTurn::WhiteTurn => PieceColor::White,
                    GameTurn::BlackTurn => PieceColor::Black,
                }
            {
                //the piece we clicked on IS the color whose turn it is
                //so: select this piece

                self.board.select_piece(piece);

                println!("Selected piece at index ({})", pos);
            } else {
                //we clicked on a piece of the other color
                //so: check if this square is a valid target for the selected piece

                //first, do we have a selected piece?
                if self.board.selected_piece_location.is_some() {
                    //we have a selected piece
                    //check if the clicked square is a valid target for the selected piece
                    if let Some(item) = self.board.selected_move_to(pos) {
                        //this is a valid target for this piece
                        //so: move the piece and capture if necessary
                        self.board.make_move(&item);
                        self.board.clear_selection();
                        if let Some(winner) = self.board.is_check_mate() {
                            println!(
                                "Checkmate! {} wins!",
                                match winner {
                                    PieceColor::White => "White",
                                    PieceColor::Black => "Black",
                                }
                            );
                        }
                        return;
                    } else {
                        println!("Invalid move to ({}) for selected piece, deselecting", pos);
                        self.board.clear_selection();
                    }
                };
            }
        } else {
            //there is not a piece in this square
            //so: check if this square is a valid target for the selected piece
            //first, do we have a selected piece?
            if self.board.selected_piece_location.is_some() {
                //we have a selected piece
                //check if the clicked square is a valid target for the selected piece
                if let Some(item) = self.board.selected_move_to(pos) {
                    //this is a valid target for this piece
                    //so: move the piece and capture if necessary
                    self.board.make_move(&item);
                    self.board.clear_selection();
                    if let Some(winner) = self.board.is_check_mate() {
                        println!(
                            "Checkmate! {} wins!",
                            match winner {
                                PieceColor::White => "White",
                                PieceColor::Black => "Black",
                            }
                        );
                    }
                    return;
                } else {
                    println!("Invalid move to ({}) for selected piece, deselecting", pos);
                    self.board.clear_selection();
                }
            };
        }
    }
}

impl<'a> Widget for BoardWidget<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let square_size = self.square_size;
        let mut total_rect: Option<Rect> = None;
        let selected_position = self.board.selected_piece_location();
        let selected_targets = self.board.selected_legal_targets().to_vec();

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
                            let square_position = Position::from_coordinates(col, row);

                            let is_light_square = (row + col) % 2 == 0;
                            let mut square_color = if is_light_square {
                                Color32::from_rgb(0, 0, 0)
                            } else {
                                Color32::from_rgb(155, 173, 183)
                            };

                            //if this square is the one with the selected piece, highlight it green
                            if let Some(selected_position) = selected_position {
                                if selected_position == square_position {
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
                            for t in &selected_targets {
                                if t.get_target_pos() == square_position {
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

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(square_size, square_size),
                                Sense::click(),
                            );
                            total_rect = Some(total_rect.map(|r| r.union(rect)).unwrap_or(rect));

                            // Draw the square
                            ui.painter().rect_filled(rect, 0.0, square_color);

                            // Draw piece as an overlay using painter and cached textures (no layout impact)
                            for (id, piece) in &self.board.pieces {
                                if *id == square_position {
                                    piece.paint(ui, rect, &mut self.board.textures);
                                }
                            }

                            // Handle click events (only for real board squares)
                            if response.clicked() {
                                self.click_on(square_position);
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
