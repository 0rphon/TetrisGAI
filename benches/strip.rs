use criterion::Criterion;

use tetris::game::*;

/// [184.92 ns 186.60 ns 188.42 ns]
pub fn stripped_data_get(c: &mut Criterion) {
    let data = pieces::tests::piece_gen_piece(pieces::PieceType::I);
    c.bench_function("strip::StrippedData::get", |b| b.iter(||
        assert!(strip::tests::stripped_data_get(&data).width>0)
    ));
}

/// [116.96 ns 118.94 ns 121.13 ns]
pub fn stripped_piece_get(c: &mut Criterion) {
    let piece = pieces::Piece::gen_random((0, 0));
    c.bench_function("strip::StrippedPiece::get", |b| b.iter(||
        assert!(strip::tests::stripped_piece_get(&piece).can_hold)
    ));
}

/// [981.61 ns 1.0011 us 1.0217 us]
pub fn stripped_board_get(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("strip::StrippedPiece::get", |b| b.iter(||
        assert!(!strip::StrippedBoard::get(&board).gameover)
    ));
}