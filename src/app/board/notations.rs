use crate::{
    Board,
    app::board::{Piece, PieceColor, piece::PieceType},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveNotation {
    pub moving_piece_color: PieceColor,
    pub moving_piece_type: PieceType,
    pub from_file: usize,
    pub from_rank: usize,
    pub is_check: bool,
    pub is_self_check: bool,
    pub is_capture: bool,
    pub to_file: usize,
    pub to_rank: usize,
}

impl MoveNotation {
    pub fn from_target(piece: &Piece, target: (usize, usize), board: &Board) -> Self {
        let mut is_capture = false;
        let mut is_check = false;

        //if target defines a square containing a piece
        if let Some(targeted_piece) = board.pieces.get(&target) {
            //if that piece is of the opposite color
            if targeted_piece.color != piece.color {
                is_capture = true;

                //if the targeted piece is a king
                if targeted_piece.piece_type == PieceType::King {
                    is_check = true;
                }
            }
        }

        Self {
            moving_piece_color: piece.color,
            moving_piece_type: piece.piece_type,
            from_file: piece.position.0,
            from_rank: piece.position.1,
            is_check,
            is_self_check: false,
            is_capture,
            to_file: target.0,
            to_rank: target.1,
        }
    }

    pub fn to_string(&self) -> String {
        let icon: String = match (self.moving_piece_color, self.moving_piece_type) {
            (PieceColor::White, PieceType::King) => "♔".to_string(),
            (PieceColor::White, PieceType::Queen) => "♕".to_string(),
            (PieceColor::White, PieceType::Rook) => "♖".to_string(),
            (PieceColor::White, PieceType::Bishop) => "♗".to_string(),
            (PieceColor::White, PieceType::Knight) => "♘".to_string(),
            (PieceColor::White, PieceType::Pawn) => "♙".to_string(),
            (PieceColor::Black, PieceType::King) => "♚".to_string(),
            (PieceColor::Black, PieceType::Queen) => "♛".to_string(),
            (PieceColor::Black, PieceType::Rook) => "♜".to_string(),
            (PieceColor::Black, PieceType::Bishop) => "♝".to_string(),
            (PieceColor::Black, PieceType::Knight) => "♞".to_string(),
            (PieceColor::Black, PieceType::Pawn) => "♟".to_string(),
        };

        format!(
            "{} {}{}{}{}{}{}",
            icon,
            Self::file_to_char(self.from_file),
            Self::rank_to_char(self.from_rank),
            if self.is_capture { "x" } else { "" },
            Self::file_to_char(self.to_file),
            Self::rank_to_char(self.to_rank),
            if self.is_check { "+" } else { "" }
        )
    }
    fn file_to_char(file: usize) -> char {
        let c = match file + 1 {
            1 => 'a',
            2 => 'b',
            3 => 'c',
            4 => 'd',
            5 => 'e',
            6 => 'f',
            7 => 'g',
            8 => 'h',
            _ => panic!("Invalid rank input"),
        };
        c
    }
    fn rank_to_char(file: usize) -> char {
        let c = match file + 1 {
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            _ => panic!("Invalid file input"),
        };
        c
    }

    pub fn targets_enemy(&self, board: &Board) -> Option<Piece> {
        let target_square = self.get_target_pos();
        if let Some(target_piece) = board.pieces.get(&target_square.to_pos()) {
            if target_piece.color != self.moving_piece_color {
                return Some(target_piece.clone());
            }
        }
        None
    }

    pub fn targets_enemy_king(&self, board: &Board) -> bool {
        if let Some(target_piece) = self.targets_enemy(board) {
            if target_piece.piece_type == PieceType::King {
                return true;
            }
        }
        return false;
    }

    pub fn apply_check_status(&mut self, board: &Board) {
        //determine if this move is targeting the opponent's king
        self.is_check = self.targets_enemy_king(board);
    }

    pub fn to_tuple(
        notation: MoveNotation,
    ) -> (
        PieceColor,
        PieceType,
        (usize, usize),
        (usize, usize),
        bool,
        bool,
    ) {
        let color = notation.moving_piece_color;
        let piece = notation.moving_piece_type;
        let from_file = notation.from_file;
        let from_rank = notation.from_rank;
        let to_file = notation.to_file;
        let to_rank = notation.to_rank;
        let is_capture = notation.is_capture;
        let is_check = notation.is_check;

        return (
            color,
            piece,
            (from_file, from_rank),
            (to_file, to_rank),
            is_capture,
            is_check,
        );
    }
    pub fn get_source_pos(&self) -> SquareNotation {
        SquareNotation {
            file: self.from_file,
            rank: self.from_rank,
        }
    }
    pub fn get_target_pos(&self) -> SquareNotation {
        SquareNotation {
            file: self.to_file,
            rank: self.to_rank,
        }
    }
    pub fn targets_square(&self, square: &SquareNotation) -> bool {
        self.to_file == square.file && self.to_rank == square.rank
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SquareNotation {
    pub file: usize,
    pub rank: usize,
}

impl SquareNotation {
    pub fn from((file, rank): (usize, usize)) -> Self {
        Self { file, rank }
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", self.file, self.rank)
    }

    pub fn to_pos(&self) -> (usize, usize) {
        (self.file, self.rank)
    }
}
