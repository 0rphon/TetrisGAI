use criterion::{criterion_group, criterion_main};

mod game;
mod pieces;
mod strip;

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
    game::board_update_shadow,
    game::board_get_speed,
    game::board_update,
    game::board_set_piece,
    game::board_update_rows,
    game::board_update_progress,
    game::board_next_piece,
    game::board_check_collision,
    
    pieces::piece_type_get_data,
    pieces::piece_gen_block,
    pieces::piece_gen_piece,
    pieces::piece_gen_random,
    pieces::piece_get_shadow,
    pieces::piece_get_rotated,
    pieces::piece_reset_rotation,
    pieces::piece_get_down,
    pieces::piece_get_left,
    pieces::piece_get_right,
    
    strip::stripped_data_get,
    strip::stripped_piece_get,
    strip::stripped_board_get,
);
criterion_main!(benches);