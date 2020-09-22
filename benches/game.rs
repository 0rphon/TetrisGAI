use criterion::{Criterion, BatchSize};
use tetris::game::*;

//ALL PUBLIC FUNCTIONS

//[4.4579 ms 4.4822 ms 4.5101 ms]
pub fn board_new_board(c: &mut Criterion) {
    c.bench_function("game::Board::new_board", |b| b.iter(|| 
        Board::new_board().unwrap()
    ));
}

//[242.61 us 248.91 us 256.22 us]
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

//[4.6526 ms 4.7083 ms 4.7672 ms]
pub fn board_reset(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::board_reset", |b| b.iter(|| 
        board.reset().unwrap()
    ));
}

//[4.6307 ms 4.7160 ms 4.8112 ms]
pub fn board_clone(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board clone", |b| b.iter(|| 
        board.reset().unwrap()
    ));
}

//[431.31 us 441.66 us 452.36 us]
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

//[2.0541 ms 2.0975 ms 2.1445 ms]
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

//down  [244.33 us 250.94 us 259.31 us]
//left  [257.27 us 267.05 us 280.36 us]
//right [250.12 us 260.30 us 276.89 us]
//rotate[252.28 us 259.72 us 269.29 us]
//drop  [1.8757 ms 1.9314 ms 1.9946 ms]
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

//[967.49 ns 985.48 ns 1.0074 us]
pub fn board_get_board(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_board", |b| b.iter(|| 
        assert!(!board.get_board().gameover)
    ));
}






//ALL PRIVATE FUNCTIONS

//[75.694 us 76.602 us 77.549 us]
pub fn board_get_highscore(c: &mut Criterion) {
    c.bench_function("game::Board::get_highscore", |b| b.iter(|| 
        assert_ne!(tests::get_highscore().unwrap(), 0)
    ));
}

//[236.32 ps 241.23 ps 247.09 ps]
pub fn board_get_speed(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_speed", |b| b.iter(|| 
        assert_eq!(tests::get_speed(&board), 48)
    ));
}

//[257.19 us 269.59 us 286.91 us]
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

//[98.677 us 104.98 us 114.28 us]
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

//[90.628 us 100.62 us 118.01 us]
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

//[88.690 us 92.206 us 97.188 us]
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

//[168.20 us 170.94 us 173.87 us]
pub fn board_next_piece(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::next_piece", |b| b.iter(||
        assert!(tests::next_piece(&mut board))
    ));
}

//[4.6519 ns 4.7201 ns 4.8039 ns]
pub fn board_check_collision(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    let piece = tests::assist_get_piece(&board);
    c.bench_function("game::Board::check_collision", |b| b.iter(|| 
        assert!(!tests::check_collision(&mut board, &piece))
    ));
}