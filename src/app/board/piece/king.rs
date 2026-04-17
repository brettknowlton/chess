use crate::{
    Board,
    app::board::{
        MoveNotation, PieceColor,
        piece::{Piece, PieceTrait},
        position::Position,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct King {
    pub color: PieceColor,
    pub position: Position,
    pub has_moved: bool,
}

impl King {

    pub fn get_soft_targets(&self, board: &Board) -> Vec<MoveNotation> {
        let mut targets = vec![];

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

            if let Some(target_position) = self.position.get_relative_pos(df, dr) {
                if let Some(piece) = board.pieces.get(&target_position) {
                    if piece.get_color() == self.color {
                        continue;
                    }
                }

                targets.push(MoveNotation::from_target(
                    &Piece::King(self.clone()),
                    target_position,
                    &board,
                ));
            }
        }
        targets
    }
}
