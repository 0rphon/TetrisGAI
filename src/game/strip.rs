#![doc(hidden)]

use super::*;

///blocks have been replaced with Vec<bools>
#[derive(Clone, Debug)]
pub struct StrippedData {
    pub data: Vec<bool>,
    pub width: usize,
    pub height: usize,
}
impl StrippedData {
    fn strip(data: &pieces::BlockData) -> Self {
        Self {
            width: data[0].len(),
            height: data.len(),
            data: data.iter().flatten().map(|cell| cell.is_some()).collect()
        }
    }
}

#[derive(Clone)]
pub struct StrippedPiece {
    pub location: (isize, isize),
    pub data: StrippedData,
    pub can_hold: bool,
}

impl StrippedPiece {
    fn get(piece: &pieces::Piece) -> Self {
        Self {
            location: piece.location,
            data: StrippedData::strip(&piece.data),
            can_hold: piece.can_hold,
        }
    }
}


///the data returned to AI from get_board()
pub struct StrippedBoard {
    pub piece: StrippedPiece,
    pub next_piece: StrippedPiece,
    pub held_piece: Option<StrippedPiece>,
    pub data:   StrippedData,
    pub score: usize,
    pub level: usize,
    pub gameover: bool,
}

impl StrippedBoard {
    pub fn get(board: &Board) -> Self {
        Self {
            piece: StrippedPiece::get(&board.piece),
            next_piece: StrippedPiece::get(&board.next_piece),
            held_piece: if let Some(held) = &board.held_piece {Some(StrippedPiece::get(held))} else {None},
            data: StrippedData::strip(&board.data),
            score: board.score,
            level: board.level,
            gameover: board.gameover,
        }
    }
}



pub mod tests {
    use super::*;

    pub fn stripped_data_get(data: &pieces::BlockData) -> StrippedData {
        StrippedData::strip(data)
    }

    pub fn stripped_piece_get(piece: &pieces::Piece) -> StrippedPiece {
        StrippedPiece::get(piece)
    }
}