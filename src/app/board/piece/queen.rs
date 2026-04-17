use crate::{
    Board,
    app::board::{
        MoveNotation, PieceColor,
        piece::{bishop::Bishop, rook::Rook},
        position::Position,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Queen {
    pub color: PieceColor,
    pub position: Position,
}

impl Queen {
    pub fn get_soft_targets(&self, board: &Board) -> Vec<MoveNotation> {
        let mut rook_targets = Rook {
            color: self.color,
            position: self.position,
            has_moved: false,
        }
        .get_soft_targets(board);

        let mut bishop_targets = Bishop {
            color: self.color,
            position: self.position,
        }
        .get_soft_targets(board);

        rook_targets.append(&mut bishop_targets);
        rook_targets
    }
}
