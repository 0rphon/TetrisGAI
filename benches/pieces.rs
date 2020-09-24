use criterion::{Criterion, BatchSize};
use tetris::game::pieces::*;

/// [11.473 us 11.731 us 12.015 us]
/// [220.79 ns 226.74 ns 233.02 ns] AFTER PIECE INDEX
/// [211.91 ns 219.50 ns 227.51 ns] AFTER MOVEMENT REWORK
/// [260.46 ns 269.39 ns 278.80 ns]
/// [55.494 ns 56.713 ns 58.157 ns] FLAT BOARD
pub fn piece_gen_piece(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let type_ = PieceType::pick_random();
    c.bench_function("pieces::Piece::gen_piece", |b| b.iter(||
        assert!(Piece::gen_piece(type_, (0, 0), &index).can_hold)
    ));
}

/// [19.365 us 19.737 us 20.147 us]
/// [203.76 ns 209.72 ns 216.68 ns] AFTER PIECE INDEX
/// [222.56 ns 229.47 ns 236.33 ns] AFTER MOVEMENT REWORK
/// [199.68 ns 203.03 ns 207.01 ns]
/// [110.91 ns 113.43 ns 116.20 ns] FLAT BOARD
pub fn piece_get_rotated(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let type_ = PieceType::pick_random();
    let piece = Piece::gen_piece(type_, (0, 0), &index);
    c.bench_function("pieces::Piece::get_rotated", move |b| b.iter(||
        assert!(piece.get_rotated().can_hold)
    ));
}

/// [15.207 us 15.688 us 16.169 us]
/// [218.59 ns 223.43 ns 228.88 ns] AFTER PIECE INDEX
/// [292.22 ns 297.75 ns 303.97 ns] AFTER MOVEMENT REWORK                   30% WORSE
/// [387.54 ns 397.09 ns 408.58 ns]
/// [80.041 ns 80.963 ns 81.936 ns] FLAT BOARD
pub fn piece_reset_rotation(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let type_ = PieceType::pick_random();
    let piece = Piece::gen_piece(type_, (0, 0), &index);
    c.bench_function("game::Board::reset_rotation", move |b| {
        b.iter_batched(
            || piece.clone(),
            |mut piece| piece.reset_rotation(&index),
            BatchSize::SmallInput
        )
    });
}

/// [10.099 us 10.308 us 10.525 us]
/// [186.41 ns 190.39 ns 194.89 ns] AFTER PIECE INDEX
/// [465.08 ps 469.66 ps 475.00 ps] AFTER MOVEMENT REWORK
/// [447.66 ps 448.84 ps 450.32 ps]
/// [470.26 ps 475.92 ps 481.92 ps] 4% SLOWER
pub fn piece_get_down(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let type_ = PieceType::pick_random();
    let piece = Piece::gen_piece(type_, (0, 0), &index);
    c.bench_function("pieces::Piece::get_down", move |b| b.iter(||
        piece.get_down()
    ));
}

/// [9.7789 us 10.057 us 10.355 us]
/// [180.30 ns 183.81 ns 187.97 ns] AFTER PIECE INDEX
/// [448.07 ps 449.57 ps 451.35 ps] AFTER MOVEMENT REWORK
pub fn piece_get_left(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let type_ = PieceType::pick_random();
    let piece = Piece::gen_piece(type_, (0, 0), &index);
    c.bench_function("pieces::Piece::get_left", move |b| b.iter(||
        piece.get_left()
    ));
}

/// [10.019 us 10.204 us 10.399 us]
/// [181.30 ns 183.73 ns 186.35 ns] AFTER PIECE INDEX
/// [461.27 ps 467.87 ps 476.54 ps] AFTER MOVEMENT REWORK
/// [456.60 ps 461.10 ps 466.07 ps] FLAT BOARD
pub fn piece_get_right(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let type_ = PieceType::pick_random();
    let piece = Piece::gen_piece(type_, (0, 0), &index);
    c.bench_function("pieces::Piece::get_right", move |b| b.iter(||
        piece.get_right()
    ));
}