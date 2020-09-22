use criterion::{criterion_group, criterion_main};

mod game;

criterion_group!(
    benches, 
    game::board_new_board, 
    game::board_try_update, 
    game::board_reset, 
    game::board_clone, 
    game::board_piece_hold, 
    game::board_piece_drop,
    game::board_move_piece, 
    game::board_get_board,
);
criterion_main!(benches);