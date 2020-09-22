use criterion::{Criterion, BatchSize};
use tetris::game::pieces::*;

/// [11.473 us 11.731 us 12.015 us]
/// [220.79 ns 226.74 ns 233.02 ns] AFTER PIECE INDEX
pub fn piece_gen_random(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    c.bench_function("pieces::Piece::gen_random", |b| b.iter(||
        assert!(Piece::gen_random((0,0), &index).can_hold)
    ));
}

/// [19.365 us 19.737 us 20.147 us]
/// [203.76 ns 209.72 ns 216.68 ns] AFTER PIECE INDEX
pub fn piece_get_rotated(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let piece = Piece::gen_random((0,0), &index);
    c.bench_function("pieces::Piece::get_rotated", move |b| b.iter(||
        assert!(piece.get_rotated().can_hold)
    ));
}

/// [15.207 us 15.688 us 16.169 us]
/// [218.59 ns 223.43 ns 228.88 ns] AFTER PIECE INDEX
pub fn piece_reset_rotation(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let piece = Piece::gen_random((0,0), &index).get_rotated();
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
pub fn piece_get_down(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let piece = Piece::gen_random((0,0), &index);
    c.bench_function("pieces::Piece::get_down", move |b| b.iter(||
        assert!(piece.get_down().can_hold)
    ));
}

/// [9.7789 us 10.057 us 10.355 us]
/// [180.30 ns 183.81 ns 187.97 ns] AFTER PIECE INDEX
pub fn piece_get_left(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let piece = Piece::gen_random((0,0), &index);
    c.bench_function("pieces::Piece::get_left", move |b| b.iter(||
        assert!(piece.get_left().can_hold)
    ));
}

/// [10.019 us 10.204 us 10.399 us]
/// [181.30 ns 183.73 ns 186.35 ns] AFTER PIECE INDEX
pub fn piece_get_right(c: &mut Criterion) {
    let index = PieceType::gen_piece_index();
    let piece = Piece::gen_random((0,0), &index);
    c.bench_function("pieces::Piece::get_right", move |b| b.iter(||
        assert!(piece.get_right().can_hold)
    ));
}