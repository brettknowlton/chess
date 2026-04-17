use crate::{
    Board,
    app::board::{
        MoveNotation, PieceColor,
        piece::{Piece, PieceTrait},
        position::Position,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pawn {
    pub color: PieceColor,
    pub position: Position,
    pub has_moved: bool,
}

impl Pawn {
    pub fn get_soft_targets(&self, board: &Board) -> Vec<MoveNotation> {
        let mut targets = vec![];

        let direction: i8 = match self.color {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };

        //pawns move differently based on color
        if let Some(in_front) = self.position.get_relative_pos(0, direction) {
            //only add forward moves if square is empty
            if board.pieces.get(&in_front).is_none() {
                //square is empty, we can target it
                targets.push(MoveNotation::from_target(
                    &Piece::Pawn(Pawn {
                        color: self.color,
                        position: self.position,
                        has_moved: self.has_moved,
                    }),
                    in_front,
                    &board,
                ));
            }
            //initial double move
            if (self.color == PieceColor::White && self.position.rank == 2)
                || (self.color == PieceColor::Black && self.position.rank == 7)
            {
                if let Some(double_forward_rank) = in_front.get_relative_pos(0, direction) {
                    if board.pieces.get(&double_forward_rank).is_none()
                        && board.pieces.get(&in_front).is_none()
                    {
                        //only add forward moves if squares are empty
                        targets.push(MoveNotation::from_target(
                            &Piece::Pawn(Pawn {
                                color: self.color,
                                position: self.position,
                                has_moved: self.has_moved,
                            }),
                            double_forward_rank,
                            &board,
                        ));
                    }
                }
            }
        }

        //captures
        if let Some(attk_left) = self.position.get_relative_pos(-1, direction) {
            if let Some(seen_piece) = board.pieces.get(&attk_left) {
                if seen_piece.get_color() != self.color {
                    targets.push(MoveNotation::from_target(
                        &Piece::Pawn(Pawn {
                            color: self.color,
                            position: self.position,
                            has_moved: self.has_moved,
                        }),
                        attk_left,
                        &board,
                    ));
                }
            }
        }
        if let Some(attk_right) = self.position.get_relative_pos(1, direction) {
            if let Some(seen_piece) = board.pieces.get(&attk_right) {
                if seen_piece.get_color() != self.color {
                    targets.push(MoveNotation::from_target(
                        &Piece::Pawn(Pawn {
                            color: self.color,
                            position: self.position,
                            has_moved: self.has_moved,
                        }),
                        attk_right,
                        &board,
                    ));
                }
            }
        }
        targets
    }
}
