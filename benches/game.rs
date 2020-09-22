use criterion::{Criterion, BatchSize};
use tetris::game::*;

//ALL PUBLIC FUNCTIONS

/// [4.7306 ms 4.7950 ms 4.8620 ms]
/// [4.5556 ms 4.5865 ms 4.6185 ms]
pub fn board_new_board(c: &mut Criterion) {
    c.bench_function("game::Board::new_board", |b| b.iter(||
        Board::new_board().unwrap()
    ));
}

/// [268.51 us 278.90 us 290.01 us]
/// [243.09 us 246.53 us 250.53 us]
pub fn board_try_update(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::try_update", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| loop {if board.try_update().unwrap() {break}},
            BatchSize::SmallInput
        )
    });
}

/// [4.4114 ms 4.4286 ms 4.4479 ms]
/// [4.6284 ms 4.6881 ms 4.7528 ms]
pub fn board_reset(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::board_reset", |b| b.iter(||
        board.reset().unwrap()
    ));
}

/// [4.4804 ms 4.5185 ms 4.5619 ms]
/// [4.4802 ms 4.5092 ms 4.5416 ms]
pub fn board_clone(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board clone", |b| b.iter(||
        board.reset().unwrap()
    ));
}

/// [412.66 us 420.61 us 429.91 us]
/// [436.11 us 448.90 us 462.57 us]
pub fn board_piece_hold(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::piece_hold", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| assert!(board.piece_hold().unwrap()),
            BatchSize::SmallInput
        )
    });
}

/// [2.0647 ms 2.1117 ms 2.1615 ms]
/// [1.8549 ms 1.8714 ms 1.8893 ms]
pub fn board_piece_drop(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::piece_drop", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| assert!(board.piece_drop().unwrap()),
            BatchSize::SmallInput
        )
    });
}

///down     [246.97 us 255.15 us 264.45 us]
///         [250.30 us 260.01 us 271.95 us]
///left     [254.94 us 264.51 us 277.15 us]
///         [271.03 us 282.76 us 298.98 us]
///right    [247.01 us 258.59 us 275.41 us]
///         [250.18 us 260.70 us 275.50 us]
///rotate   [242.39 us 244.18 us 246.18 us]
///         [265.77 us 278.38 us 296.67 us]
///drop     [1.8852 ms 1.9362 ms 1.9909 ms]
///         [1.8699 ms 1.9170 ms 1.9683 ms]
///only moves down left right, not rotate or drop
pub fn board_move_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();

    let mut group = c.benchmark_group("game::Board::move_piece");
    let moves = [Move::Down, Move::Left, Move::Right, Move::Rotate, Move::Drop];

    for input in moves.iter() {
        group.bench_with_input(format!("{:?}", input), &(input, board.clone()), move |b, (input, board)| {
            b.iter_batched(
                || (board.clone(), input),
                |(mut board, input)| assert!(board.move_piece(**input)),
                BatchSize::SmallInput
            )
        });
    }
}

/// [1.5545 us 1.5783 us 1.6052 us]
/// [1.5177 us 1.5509 us 1.5867 us]
pub fn board_get_board(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_board", |b| b.iter(||
        assert!(!board.get_board().gameover)
    ));
}






//ALL PRIVATE FUNCTIONS

/// [75.015 us 76.457 us 78.058 us]
/// [74.382 us 75.214 us 76.237 us]
pub fn board_get_highscore(c: &mut Criterion) {
    c.bench_function("game::Board::get_highscore", |b| b.iter(||
        assert_ne!(tests::get_highscore().unwrap(), 0)
    ));
}

/// [234.05 ps 238.58 ps 244.05 ps]
/// [231.51 ps 234.46 ps 237.96 ps]
pub fn board_get_speed(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_speed", |b| b.iter(||
        assert_eq!(tests::get_speed(&board), 48)
    ));
}

/// [249.32 us 258.97 us 272.82 us]
/// [251.90 us 261.43 us 276.19 us]
pub fn board_update(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::update", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| tests::update(&mut board),
            BatchSize::SmallInput
        )
    });
}

/// [94.047 us 101.48 us 114.51 us]
/// [91.319 us 99.712 us 115.03 us]
pub fn board_set_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::set_piece", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| tests::set_piece(&mut board),
            BatchSize::SmallInput
        )
    });
}

/// [94.642 us 101.99 us 115.20 us]
/// [91.583 us 101.75 us 118.58 us]
pub fn board_update_rows(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::update_rows", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| tests::update_rows(&mut board),
            BatchSize::SmallInput
        )
    });
}

/// [93.794 us 101.69 us 116.04 us]
/// [94.974 us 107.30 us 128.64 us]
pub fn board_update_progress(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::update_progress", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| tests::update_progress(&mut board, vec!(1, 2, 3)),
            BatchSize::SmallInput
        )
    });
}

/// [168.04 us 172.53 us 177.48 us]
/// [166.42 us 170.07 us 174.58 us]
pub fn board_next_piece(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::next_piece", |b| b.iter(||
        assert!(tests::next_piece(&mut board))
    ));
}

/// [12.204 ns 12.375 ns 12.571 ns]
/// [13.522 ns 13.671 ns 13.830 ns]
pub fn board_check_collision(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    let piece = tests::assist_get_piece(&board);
    c.bench_function("game::Board::check_collision", |b| b.iter(||
        assert!(!tests::check_collision(&mut board, &piece))
    ));
}