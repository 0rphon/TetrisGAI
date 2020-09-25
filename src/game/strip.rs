#![doc(hidden)]

use super::*;

///the data returned to AI from get_board()
pub struct StrippedBoard {
    pub piece: pieces::Piece,
    pub next_piece: pieces::Piece,
    pub held_piece: Option<pieces::Piece>,
    pub data:   Vec<bool>,
    pub score: usize,
    pub level: usize,
    pub gameover: bool,
}

impl StrippedBoard {
    pub fn get(board: &Board) -> Self {
        Self {
            piece: board.piece.clone(),
            next_piece: pieces::Piece::gen_piece(board.next_piece, board.spawn, &board.piece_index),
            held_piece: board.held_piece.clone(),
            data: board.data.iter().map(|cell| cell.is_some()).collect(),
            score: board.score,
            level: board.level,
            gameover: board.gameover,
        }
    }
}