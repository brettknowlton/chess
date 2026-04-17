use crate::{
    Board,
    app::board::{
        MoveNotation, PieceColor,
        piece::{Piece, PieceTrait},
        position::Position,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rook {
    pub color: PieceColor,
    pub position: Position,
    pub has_moved: bool,
}

impl Rook {
    pub fn get_soft_targets(&self, board: &Board) -> Vec<MoveNotation> {
        let mut targets = vec![];

        //horizontal and vertical lines until blocked
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for dir in directions.iter() {
            let (df, dr) = *dir;
            let mut step = 1;
            loop {
                if let Some(target_position) = self.position.get_relative_pos(df * step, dr * step)
                {
                    //the target square is on the board, check if it's occupied
                    if let Some(seen_piece) = board.pieces.get(&target_position) {
                        //the target square is occupied
                        if seen_piece.get_color() != self.color {
                            //the target square is occupied by an enemy piece, we can target it but not look past it
                            targets.push(MoveNotation::from_target(
                                &Piece::Rook(self.clone()),
                                target_position,
                                &board,
                            ));
                        }
                        //if the piece is the same color, we can't target it, and either way we can't look past it so we stop here
                        break; //blocked by any piece
                    } else {
                        //the target square is empty, so we can target it and keep looking
                        targets.push(MoveNotation::from_target(
                            &Piece::Rook(self.clone()),
                            target_position,
                            &board,
                        ));
                        step += 1;
                    }
                } else {
                    //the target square is off the board, stop looking in this direction
                    break;
                }
            }
        }
        targets
    }

    pub fn get_relative_targets(position: Position, board: &Board) -> Vec<MoveNotation> {
        let mut targets = vec![];

        let dummy_rook = Rook {
            color: PieceColor::White,
            position,
            has_moved: false,
        };
        targets.append(dummy_rook.get_soft_targets(board).as_mut());

        targets
    }
}
