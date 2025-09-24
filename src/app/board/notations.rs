use crate::{
    Board,
    app::board::{Piece, PieceColor, piece::PieceType},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveNotation {
    pub piece_color: PieceColor,
    pub piece_type: PieceType,
    pub from_file: usize,
    pub from_rank: usize,
    pub check_opt: char,
    pub is_capture: bool,
    pub to_file: usize,
    pub to_rank: usize,
}

impl MoveNotation {
    pub fn from_target(piece: &Piece, target: (usize, usize), board: &Board) -> Self {

        let is_capture = board.pieces.get(&target).is_some();

        let mut check_opt = ' ';
        let checks = board
            .simulate_move(&Self {
                piece_color: piece.color,
                piece_type: piece.piece_type,
                from_file: piece.position.0,
                from_rank: piece.position.1,
                check_opt: check_opt,
                is_capture: is_capture,
                to_file: target.0,
                to_rank: target.1,
            })
            .is_in_check();

        let (wc, bc) = checks;
        if (wc && piece.color == PieceColor::Black) || (bc && piece.color == PieceColor::White) {
            check_opt = '+';
        } else if (wc && piece.color == PieceColor::White) || (bc && piece.color == PieceColor::Black) {
            check_opt = '-';
        }
        Self {
            piece_color: piece.color,
            piece_type: piece.piece_type,
            from_file: piece.position.0,
            from_rank: piece.position.1,
            check_opt,
            is_capture,
            to_file: target.0,
            to_rank: target.1,
        }
    }

    pub fn to_string(&self) -> String {
        let piece_color = match self.piece_color {
            PieceColor::White => "W",
            PieceColor::Black => "B",
        };
        let piece_type = match self.piece_type {
            PieceType::Pawn => "P",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K",
        };
        format!(
            "{}{}{}{}{}{}{}{}",
            piece_color,
            piece_type,
            self.from_rank,
            self.from_file,
            if self.is_capture { "x" } else { "" },
            self.to_file,
            self.to_rank,
            self.check_opt
        )
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
}