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
    game::board_get_highscore,
    game::board_get_speed,
    game::board_update,
    game::board_set_piece,
    game::board_update_rows,
    game::board_update_progress,
    game::board_next_piece,
    game::board_check_collision,
);
criterion_main!(benches);