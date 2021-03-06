use criterion::{Criterion, BatchSize};
use tetris::game::*;

//ALL PUBLIC FUNCTIONS

/// [4.5556 ms 4.5865 ms 4.6185 ms]
/// [4.2711 ms 4.2945 ms 4.3187 ms]         AFTER PIECE INDEX
/// [4.4802 ms 4.5378 ms 4.5997 ms]         AFTER MOVEMENT REWORK                   SLIGHTLY WORSE
/// [4.6067 ms 4.6855 ms 4.7773 ms]
/// [4.4406 ms 4.4700 ms 4.5009 ms]         AFTER FLAT BOARD
pub fn board_new_board(c: &mut Criterion) {
    c.bench_function("game::Board::new_board", |b| b.iter(||
        Board::new_board().unwrap()
    ));
}

/// [243.09 us 246.53 us 250.53 us]
/// [96.939 us 99.652 us 102.97 us]         AFTER REMOVE EXCESSIVE SHADOW UPDATES
/// [83.872 us 89.655 us 99.904 us]         AFTER PIECE INDEX
/// [85.663 us 87.746 us 90.275 us]
/// [73.339 us 74.609 us 75.834 us]         AFTER FLAT BOARD
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
/// [4.4033 ms 4.4805 ms 4.5663 ms]     AFTER PIECE INDEX
/// [4.3125 ms 4.3413 ms 4.3727 ms]     AFTER MOVEMENT REWORK               SLIGHTLY WORSE
/// [4.4248 ms 4.4675 ms 4.5125 ms]
pub fn board_reset(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::board_reset", |b| b.iter(||
        board.reset().unwrap()
    ));
}

/// [4.4802 ms 4.5092 ms 4.5416 ms]
/// [4.4159 ms 4.4705 ms 4.5287 ms]     AFTER PIECE INDEX
/// [4.2473 ms 4.2721 ms 4.3035 ms]
pub fn board_clone(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board clone", |b| b.iter(||
        board.reset().unwrap()
    ));
}

/// [436.11 us 448.90 us 462.57 us]
/// [99.524 us 103.24 us 107.97 us]     AFTER PIECE INDEX
/// [83.416 us 89.872 us 101.57 us]     AFTER MOVEMENT REWORK
/// [86.353 us 90.221 us 97.233 us]
pub fn board_hold_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::hold_piece", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| assert!(board.hold_piece().unwrap()),
            BatchSize::SmallInput
        )
    });
}

/// [1.8549 ms 1.8714 ms 1.8893 ms]
/// [387.46 us 400.85 us 416.44 us]     AFTER REMOVE EXCESSIVE SHADOW UPDATES
/// [100.94 us 108.82 us 121.97 us]     AFTER PIECE INDEX
/// [90.139 us 98.531 us 112.39 us]     AFTER MOVEMENT REWORK
/// [95.755 us 99.388 us 103.91 us]
/// [77.869 us 83.770 us 94.230 us]     FLAT BOARD
pub fn board_drop_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::drop_piece", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| assert!(board.drop_piece().unwrap()),
            BatchSize::SmallInput
        )
    });
}


/// [265.77 us 278.38 us 296.67 us]
/// [271.76 us 284.51 us 303.10 us]     AFTER REMOVE SHADOW UPDATES
/// [97.559 us 99.446 us 101.42 us]     AFTER PIECE INDEX
/// [89.704 us 97.779 us 112.03 us]     AFTER MOVEMENT REWORK
/// [92.144 us 99.126 us 111.76 us]
pub fn board_rotate_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::rotate_piece", move |b| {
        b.iter_batched(
            || board.clone(),
            |mut board| assert!(board.rotate_piece()),
            BatchSize::SmallInput
        )
    });
}

///down     [250.30 us 260.01 us 271.95 us]
///         [108.35 us 116.07 us 129.14 us]         AFTER REMOVE SHADOW UPDATES
///         [90.958 us 96.468 us 104.92 us]         AFTER PIECE INDEX
///         [85.634 us 90.497 us 99.015 us]         AFTER MOVEMENT REWORK
///left     [271.03 us 282.76 us 298.98 us]
///         [263.59 us 271.21 us 279.54 us]
///         [96.954 us 101.59 us 109.06 us]
///         [88.074 us 92.606 us 99.227 us]
///right    [250.18 us 260.70 us 275.50 us]
///         [268.70 us 275.47 us 282.03 us]
///         [93.179 us 96.728 us 102.04 us]
///         [92.561 us 94.613 us 96.663 us]
///only moves down left right, not rotate or drop
pub fn board_move_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();

    let mut group = c.benchmark_group("game::Board::move_piece");
    let moves = [Move::Down, Move::Left, Move::Right];

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
/// [971.78 ns 984.94 ns 998.14 ns]     AFTER PIECE INDEX
/// [940.94 ns 961.66 ns 983.11 ns]     AFTER MOVEMENT REWORK
/// [1.0103 us 1.0371 us 1.0659 us]
/// [160.39 ns 163.08 ns 166.38 ns]     FLAT BOARD
pub fn board_get_board(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_board", |b| b.iter(||
        assert!(!board.get_board().gameover)
    ));
}






//ALL PRIVATE FUNCTIONS

/// [74.382 us 75.214 us 76.237 us]
/// [72.556 us 73.141 us 73.845 us]     AFTER MOVEMENT REWORK
/// [76.028 us 84.301 us 102.26 us]
pub fn board_get_highscore(c: &mut Criterion) {
    c.bench_function("game::Board::get_highscore", |b| b.iter(||
        assert_ne!(tests::get_highscore().unwrap(), 0)
    ));
}

/// [229.48 us 233.69 us 238.13 us]
/// [84.463 us 90.606 us 101.82 us]         AFTER PIECE INDEX
/// [89.121 us 97.625 us 112.19 us]         AFTER MOVEMENT REWORK
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
/// [224.21 ps 224.78 ps 225.45 ps] AFTER PIECE INDEX
/// [226.53 ps 229.50 ps 233.42 ps] AFTER MOVEMENT REWORK
/// [243.73 ps 250.42 ps 257.67 ps]
/// [228.82 ps 230.75 ps 232.91 ps] FLAT BOARD
pub fn board_get_speed(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_speed", |b| b.iter(||
        assert_eq!(tests::get_speed(&board), 48)
    ));
}

/// [251.90 us 261.43 us 276.19 us]
/// [82.174 us 85.227 us 88.467 us] WHEN YOU SKIP MOVE DOWN CHECK?? this doesnt add up
/// [103.24 us 112.33 us 128.60 us] AFTER REMOVE EXCESSIVE SHADOW CHECKS
/// [85.238 us 89.368 us 96.540 us] AFTER PIECE INDEX
/// [85.718 us 93.288 us 106.39 us] AFTER MOVEMENT REWORK
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
/// [88.113 us 91.728 us 96.726 us] AFTER PIECE INDEX
/// [88.423 us 93.115 us 101.65 us] AFTER MOVEMENT REWORK
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
/// [81.821 us 90.325 us 105.80 us] AFTER PIECE INDEX
/// [79.682 us 87.935 us 103.99 us]
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
/// [87.130 us 89.183 us 91.330 us] AFTER PIECE INDEX
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
/// [4.5054 us 4.5787 us 4.6587 us] AFTER PIECE INDEX
/// [448.44 ns 459.70 ns 471.48 ns] AFTER MOVEMENT REWORK
/// [645.91 ns 649.00 ns 652.46 ns] FLAT BOARD                      34% SLOWER
pub fn board_next_piece(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::next_piece", |b| b.iter(||
        assert!(tests::next_piece(&mut board))
    ));
}

/// [13.522 ns 13.671 ns 13.830 ns]
/// [7.6780 ns 7.8120 ns 7.9486 ns] AFTER PIECE INDEX
/// [16.031 ns 16.158 ns 16.307 ns] AFTER MOVEMENT REWORK                   WORSE BT 111%
/// [11.055 ns 11.216 ns 11.416 ns]
/// [24.533 ns 24.597 ns 24.677 ns] FLAT BOARD                              122% SLOWER???
pub fn board_check_collision(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    let piece = tests::assist_get_piece(&board);
    c.bench_function("game::Board::check_collision", |b| b.iter(||
        assert!(!tests::check_collision(&mut board, &piece, piece.location))
    ));
}