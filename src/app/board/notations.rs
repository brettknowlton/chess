use crate::{
    Board,
    app::board::{
        Piece, PieceColor,
        piece::{PieceTrait, PieceType},
        position::Position,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveNotation {
    pub moving_piece_color: PieceColor,
    pub moving_piece_type: PieceType,
    pub moving_from_position: Position,
    pub is_check: bool,
    pub is_self_check: bool,
    pub is_capture: bool,
    pub moving_to_position: Position,
}

impl MoveNotation {
    pub fn from_target(piece: &Piece, target: Position, board: &Board) -> Self {
        let mut is_capture = false;

        //if target defines a square containing a piece
        if let Some(targeted_piece) = board.pieces.get(&target) {
            //if that piece is of the opposite color
            if targeted_piece.get_color() != piece.get_color() {
                is_capture = true;
            }
        }

        Self {
            moving_piece_color: piece.get_color(),
            moving_piece_type: piece.get_type(),
            moving_from_position: piece.get_position(),
            is_check: false,
            is_self_check: false,
            is_capture,
            moving_to_position: target,
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
            "{} {}{}{}{}",
            icon,
            self.moving_from_position.to_string(),
            if self.is_capture { "x" } else { "" },
            self.moving_to_position.to_string(),
            if self.is_check { "+" } else { "" }
        )
    }

    pub fn targets_enemy(&self, board: &Board, color: PieceColor) -> Option<Piece> {
        let target_square = self.get_target_pos();
        if let Some(target_piece) = board.pieces.get(&target_square) {
            if target_piece.get_color() != color {
                return Some(target_piece.clone());
            }
        }
        None
    }

    pub fn targets_enemy_king(&self, board: &Board, color: PieceColor) -> bool {
        if let Some(target_piece) = self.targets_enemy(board, color) {
            if target_piece.get_type() == PieceType::King && target_piece.get_color() != color {
                return true;
            }
        }
        return false;
    }

    pub fn apply_check_status(&mut self, board: &Board) {
        //determine if this move is targeting the opponent's king
        self.is_check = self.targets_enemy_king(board, self.moving_piece_color);
    }

    pub fn to_tuple(
        notation: MoveNotation,
    ) -> (PieceColor, PieceType, Position, Position, bool, bool) {
        let color = notation.moving_piece_color;
        let piece = notation.moving_piece_type;
        let from_position = notation.moving_from_position;
        let to_position = notation.moving_to_position;
        let is_capture = notation.is_capture;
        let is_check = notation.is_check;

        return (
            color,
            piece,
            from_position,
            to_position,
            is_capture,
            is_check,
        );
    }

    pub fn get_source_pos(&self) -> Position {
        self.moving_from_position
    }

    pub fn get_target_pos(&self) -> Position {
        self.moving_to_position
    }

    pub fn targets_square(&self, square: &Position) -> bool {
        self.moving_to_position == *square
    }
}
