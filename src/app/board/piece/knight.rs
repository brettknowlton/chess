use crate::{
    Board,
    app::board::{
        MoveNotation, Piece, PieceColor,
        piece::{PieceTrait, Position},
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Knight {
    pub color: PieceColor,
    pub position: Position,
}

impl Knight {
    pub fn get_soft_targets(&self, board: &Board) -> Vec<MoveNotation> {
        let mut targets = vec![];

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
            if let Some(target_position) = self.position.get_relative_pos(km.0, km.1) {
                if let Some(piece) = board.pieces.get(&target_position) {
                    if piece.get_color() == self.color {
                        continue; //can't target pieces of the same color
                    }
                }
                targets.push(MoveNotation::from_target(
                    &Piece::Knight(self.clone()),
                    target_position,
                    &board,
                ));
            }
        }

        targets
    }
}
