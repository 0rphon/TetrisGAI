use criterion::Criterion;

use tetris::game::*;

/// [553.08 ns 560.65 ns 569.33 ns] AFTER PIECE INDEX
pub fn stripped_data_strip_board(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    let data = tests::assist_get_board_data(&board);
    c.bench_function("strip::StrippedData::strip_board", |b| b.iter(||
        assert!(strip::tests::stripped_data_strip_board(&data).width>0)
    ));
}

/// [176.16 ns 177.27 ns 178.57 ns] AFTER PIECE INDEX
pub fn stripped_data_strip_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    let data = tests::assist_get_piece(&board).data;
    c.bench_function("strip::StrippedData::strip_piece", |b| b.iter(||
        assert!(strip::tests::stripped_data_strip_piece(&data).width>0)
    ));
}

/// [116.96 ns 118.94 ns 121.13 ns]
/// [192.24 ns 197.42 ns 203.06 ns] AFTER PIECE INDEX                                       WORSE probably due to all the derefs. should be easily fixed when 1d vecs added
pub fn stripped_piece_get(c: &mut Criterion) {
    let index = pieces::PieceType::gen_piece_index();
    let piece = pieces::Piece::gen_random((0, 0), &index);
    c.bench_function("strip::StrippedPiece::get", |b| b.iter(||
        assert!(strip::tests::stripped_piece_get(&piece).can_hold)
    ));
}

/// [981.61 ns 1.0011 us 1.0217 us]
/// [960.94 ns 981.28 ns 1.0044 us] AFTER PIECE INDEX
pub fn stripped_board_get(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("strip::StrippedBoard::get", |b| b.iter(||
        assert!(!strip::StrippedBoard::get(&board).gameover)
    ));
}