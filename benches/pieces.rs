use criterion::{Criterion, BatchSize};
use tetris::game::pieces::*;

/// [230.71 ns 235.57 ns 240.77 ns]
pub fn piece_type_get_data(c: &mut Criterion) {
    c.bench_function("pieces::PieceType::get_data", |b| b.iter(||
        assert!(tests::piece_type_get_data(&PieceType::I).0[1][0])
    ));
}

/// [2.7642 us 2.8105 us 2.8635 us]
pub fn piece_gen_block(c: &mut Criterion) {
    c.bench_function("pieces::Piece::gen_block", |b| b.iter(||
        assert_eq!(tests::piece_gen_block([0xFF;4]).width, 32)
    ));
}

/// [11.236 us 11.435 us 11.666 us]
pub fn piece_gen_piece(c: &mut Criterion) {
    c.bench_function("pieces::Piece::gen_piece", |b| b.iter(||
        assert!(tests::piece_gen_piece(PieceType::I)[1][0].is_some())
    ));
}

/// [11.473 us 11.731 us 12.015 us]
pub fn piece_gen_random(c: &mut Criterion) {
    c.bench_function("pieces::Piece::gen_random", |b| b.iter(||
        assert!(Piece::gen_random((0,0)).can_hold)
    ));
}

/// [20.936 us 21.444 us 22.047 us]
pub fn piece_get_shadow(c: &mut Criterion) {
    let piece = Piece::gen_random((0,0));
    c.bench_function("pieces::Piece::get_shadow", move |b| b.iter(||
        assert!(piece.get_shadow().can_hold)
    ));
}

/// [19.365 us 19.737 us 20.147 us]
pub fn piece_get_rotated(c: &mut Criterion) {
    let piece = Piece::gen_random((0,0));
    c.bench_function("pieces::Piece::get_rotated", move |b| b.iter(||
        assert!(piece.get_rotated().can_hold)
    ));
}

/// [15.207 us 15.688 us 16.169 us]
pub fn piece_reset_rotation(c: &mut Criterion) {
    let piece = Piece::gen_random((0,0)).get_rotated();
    c.bench_function("game::Board::reset_rotation", move |b| {
        b.iter_batched(
            || piece.clone(),
            |mut piece| piece.reset_rotation(),
            BatchSize::SmallInput
        )
    });
}

/// [10.099 us 10.308 us 10.525 us]
pub fn piece_get_down(c: &mut Criterion) {
    let piece = Piece::gen_random((0,0));
    c.bench_function("pieces::Piece::get_down", move |b| b.iter(||
        assert!(piece.get_down().can_hold)
    ));
}

/// [9.7789 us 10.057 us 10.355 us]
pub fn piece_get_left(c: &mut Criterion) {
    let piece = Piece::gen_random((0,0));
    c.bench_function("pieces::Piece::get_left", move |b| b.iter(||
        assert!(piece.get_left().can_hold)
    ));
}

/// [10.019 us 10.204 us 10.399 us]
pub fn piece_get_right(c: &mut Criterion) {
    let piece = Piece::gen_random((0,0));
    c.bench_function("pieces::Piece::get_right", move |b| b.iter(||
        assert!(piece.get_right().can_hold)
    ));
}