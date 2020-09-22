use criterion::{Criterion, BatchSize};
use tetris::game::*;

//[4.5737 ms 4.6145 ms 4.6588 ms]
pub fn board_new_board(c: &mut Criterion) {
    c.bench_function("game::Board::new_board", |b| b.iter(|| 
        Board::new_board().unwrap()
    ));
}

//[315.81 us 336.88 us 359.26 us]
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

//[4.5192 ms 4.5735 ms 4.6394 ms]
pub fn board_reset(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board::board_reset", |b| b.iter(|| 
        board.reset().unwrap()
    ));
}

//[4.6026 ms 4.6474 ms 4.6968 ms]
pub fn board_clone(c: &mut Criterion) {
    let mut board = Board::new_board().unwrap();
    c.bench_function("game::Board deep clone", |b| b.iter(|| 
        board.reset().unwrap()
    ));
}


//[396.57 us 401.76 us 407.41 us]
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

//[4.6026 ms 4.6474 ms 4.6968 ms]
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

//down  [4.6026 ms 4.6474 ms 4.6968 ms]
//left  [265.98 us 275.54 us 286.84 us]
//right [264.43 us 275.10 us 289.26 us]
//rotate[258.89 us 268.01 us 282.04 us]
///only moves down left right, not rotate or drop
pub fn board_move_piece(c: &mut Criterion) {
    let board = Board::new_board().unwrap();

    let mut group = c.benchmark_group("game::Board::move_piece");
    let moves = [Move::Down, Move::Left, Move::Right, Move::Rotate];

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


//[897.17 ns 905.10 ns 913.43 ns]
pub fn board_get_board(c: &mut Criterion) {
    let board = Board::new_board().unwrap();
    c.bench_function("game::Board::get_board", |b| b.iter(|| 
        assert!(!board.get_board().gameover)
    ));
}