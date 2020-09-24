use criterion::Criterion;

use tetris::game::*;

/// [981.61 ns 1.0011 us 1.0217 us]
/// [960.94 ns 981.28 ns 1.0044 us] AFTER PIECE INDEX
/// [936.66 ns 956.15 ns 977.69 ns] AFTER MOVEMENT REWORK
/// [973.08 ns 991.58 ns 1.0135 us]
/// [174.73 ns 178.13 ns 181.66 ns] FLAT BOARD
pub fn stripped_board_get(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("strip::StrippedBoard::get", |b| b.iter(||
        assert!(!strip::StrippedBoard::get(&board).gameover)
    ));
}