use criterion::{Criterion, BatchSize};
use tetris::game::*;

//ALL PUBLIC FUNCTIONS

/// [4.5556 ms 4.5865 ms 4.6185 ms]
pub fn board_new_board(c: &mut Criterion) {
    c.bench_function("game::Board::new_board", |b| b.iter(||
        Board::new_board().unwrap()
    ));
}

/// [243.09 us 246.53 us 250.53 us]
/// [96.939 us 99.652 us 102.97 us]         AFTER REMOVE EXCESSIVE SHADOW UPDATES
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

/// [4.6284 ms 4.6881 ms 4.7528 ms]
pub fn board_reset(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::board_reset", |b| b.iter(||
        board.reset().unwrap()
    ));
}

/// [4.4802 ms 4.5092 ms 4.5416 ms]
pub fn board_clone(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board clone", |b| b.iter(||
        board.reset().unwrap()
    ));
}

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

/// [1.8549 ms 1.8714 ms 1.8893 ms]
/// [387.46 us 400.85 us 416.44 us]     //AFTER REMOVE EXCESSIVE SHADOW UPDATES
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

///down     [250.30 us 260.01 us 271.95 us]
///         [108.35 us 116.07 us 129.14 us]         AFTER REMOVE SHADOW UPDATES
///left     [271.03 us 282.76 us 298.98 us]
///         [263.59 us 271.21 us 279.54 us]
///right    [250.18 us 260.70 us 275.50 us]
///         [268.70 us 275.47 us 282.03 us]
///rotate   [265.77 us 278.38 us 296.67 us]
///         [271.76 us 284.51 us 303.10 us]
///drop     [1.8699 ms 1.9170 ms 1.9683 ms]
///         [232.91 us 247.01 us 267.06 us]
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

/// [1.5177 us 1.5509 us 1.5867 us]
pub fn board_get_board(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_board", |b| b.iter(||
        assert!(!board.get_board().gameover)
    ));
}






//ALL PRIVATE FUNCTIONS

/// [74.382 us 75.214 us 76.237 us]
pub fn board_get_highscore(c: &mut Criterion) {
    c.bench_function("game::Board::get_highscore", |b| b.iter(||
        assert_ne!(tests::get_highscore().unwrap(), 0)
    ));
}

/// [229.48 us 233.69 us 238.13 us]
pub fn board_update_shadow(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::update_shadow", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| tests::update_shadow(&mut board),
            BatchSize::SmallInput
        )
    });
}

/// [231.51 ps 234.46 ps 237.96 ps]
pub fn board_get_speed(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_speed", |b| b.iter(||
        assert_eq!(tests::get_speed(&board), 48)
    ));
}

/// [251.90 us 261.43 us 276.19 us]
/// [82.174 us 85.227 us 88.467 us] WHEN YOU SKIP MOVE DOWN CHECK?? this doesnt add up
/// [103.24 us 112.33 us 128.60 us] AFTER REMOVE EXCESSIVE SHADOW CHECKS
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

/// [166.42 us 170.07 us 174.58 us]
pub fn board_next_piece(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::next_piece", |b| b.iter(||
        assert!(tests::next_piece(&mut board))
    ));
}

/// [13.522 ns 13.671 ns 13.830 ns]
pub fn board_check_collision(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    let piece = tests::assist_get_piece(&board);
    c.bench_function("game::Board::check_collision", |b| b.iter(||
        assert!(!tests::check_collision(&mut board, &piece))
    ));
}