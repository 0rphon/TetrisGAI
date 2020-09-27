//#![windows_subsystem = "windows"]         //UNCOMMENT FOR RELEASE
#![feature(test)]
pub mod game;
pub use game::{Board, Move};
mod ai;
mod train;

use dynerr::*;
use engine;

///the target fps
const TARGET_FPS: u64 = 60;
const GAME_TITLE: &str = "Tetris";

pub fn run(train: bool, auto_loop: bool) {
    if train {
        check!(train::train());
        return
    }

    //UNTRAINED 3 : 0.500 : 0.500 : 0.250 : 0.750 : 0.000 : 0.500 : 3.500 : 2 : 0.750
    //let parameters = ai::AiParameters {
    //    min_lines_to_clear:             3,
    //    lines_cleared_importance:       0.500,
    //    points_scored_importance:       0.500,
    //    piece_depth_importance:         0.250,
    //    max_height_importance:          0.750,
    //    avg_height_importance:          0.000,
    //    height_variation_importance:    0.500,
    //    current_holes_importance:       3.500,
    //    max_pillar_height:              2,
    //    current_pillars_importance:     0.750,
    //};
    
    //   ???  |    ?? |    ??? | 2   : 0.919   : 0.006   : 0.572   : 0.049   : 0.143   : 0.012   : 0.995   : 0   : 0.392        3.08M   LESS EFFICIENT
    //392860  |    8  |    227 | 2   : 0.939   : 0.004   : 0.240   : 0.002   : 0.102   : 0.024   : 0.920   : 0   : 0.200        1.11M
    //693944  |    6  |    196 | 4   : 0.79000 : 0.00100 : 0.14900 : 0.05000 : 0.09200 : 0.01900 : 0.63200 : 1   : 0.06600
    //1189945 |   18  |    485 | 4   : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0   : 0.06300
    //1145311 |   21  |    556 | 4.0 : 0.91910 : 0.00000 : 0.56862 : 0.09960 : 0.32070 : 0.06357 : 1.00564 : 1.0 : 0.38466      2.11M lvl 30~
    let parameters = ai::AiParameters {
        min_lines_to_clear:             4.0,
        lines_cleared_importance:       0.91910,
        points_scored_importance:       0.00000,
        piece_depth_importance:         0.56862,
        max_height_importance:          0.09960,
        avg_height_importance:          0.32070,
        height_variation_importance:    0.06357,
        current_holes_importance:       1.00000,
        max_pillar_height:              1.0,
        current_pillars_importance:     0.38466,
    };

    let mut board = check!(Board::new_board());
    let mut ai_radio = None;

    let mut screen = engine::drawing::Screen::new(
        board.screen_dim.0,
        board.screen_dim.1
    );
    let mut fpslock = engine::game::FpsLock::create_lock(TARGET_FPS);
    let event_loop = engine::game::EventLoop::new();
    let mut input = engine::game::WinitInputHelper::new();
    let mut window = engine::game::Window::init(
        GAME_TITLE,
        board.screen_dim.0,
        board.screen_dim.1,
        &event_loop
    );

    event_loop.run(move |event, _, control_flow| {
        fpslock.start_frame();
        if let engine::game::Event::RedrawRequested(_) = event {
            screen.wipe();
            board.draw(&mut screen);
            if ai_radio.is_some() {
                screen.draw_text((0,0), fpslock.get_fps(), 16.0, &[0xFF;4], engine::drawing::DEBUG_FONT);
            }
            screen.flatten(window.pixels.get_frame());
            window.pixels.render().unwrap();
            fpslock.end_frame();
        }

        if input.update(&event) {
            if input.key_pressed(engine::game::VirtualKeyCode::P) {
                ai_radio = {
                    match ai_radio {
                        Some(_) => None,
                        None => Some(ai::start(parameters.clone(), false)),     //turn on to debug
                    }
                }
            }
            if ai_radio.is_some() {
                if let Some(ai_input) = check!(ai_radio.as_ref().unwrap().get_input()) {
                    match ai_input {
                        //ai::Move::Down      => {board.move_piece(Move::Down);},
                        ai::Move::Left      => {board.move_piece(Move::Left);},
                        ai::Move::Right     => {board.move_piece(Move::Right);},
                        ai::Move::Rotate    => {board.rotate_piece();}
                        ai::Move::Drop      => {check!(board.drop_piece());},
                        ai::Move::Hold      => {check!(board.hold_piece());},
                        ai::Move::Restart   => if auto_loop {check!(board.reset())},
                        ai::Move::None      => {},
                    }
                }
            } else {
                if input.key_pressed(engine::game::VirtualKeyCode::A)
                || input.key_pressed(engine::game::VirtualKeyCode::Left)
                {board.move_piece(Move::Left);}

                if input.key_pressed(engine::game::VirtualKeyCode::S)
                || input.key_pressed(engine::game::VirtualKeyCode::Down)
                {board.move_piece(Move::Down);}


                if input.key_pressed(engine::game::VirtualKeyCode::D)
                || input.key_pressed(engine::game::VirtualKeyCode::Right)
                {board.move_piece(Move::Right);}

                if input.key_pressed(engine::game::VirtualKeyCode::W)
                || input.key_pressed(engine::game::VirtualKeyCode::R)
                || input.key_pressed(engine::game::VirtualKeyCode::X)
                || input.key_pressed(engine::game::VirtualKeyCode::Up)
                {board.rotate_piece();}


                if input.key_pressed(engine::game::VirtualKeyCode::F)
                ||input.key_pressed(engine::game::VirtualKeyCode::C)
                {check!(board.hold_piece());}

                if input.key_pressed(engine::game::VirtualKeyCode::Space)
                {check!(board.drop_piece());}
            }
            if input.key_pressed(engine::game::VirtualKeyCode::Return)
            || input.key_pressed(engine::game::VirtualKeyCode::NumpadEnter)
            || input.key_pressed(engine::game::VirtualKeyCode::Space) && board.gameover
                {check!(board.reset())}

            if input.key_pressed(engine::game::VirtualKeyCode::Escape) || input.quit() {
                *control_flow = engine::game::ControlFlow::Exit;
                if ai_radio.is_some() {
                    check!(ai_radio.as_mut().unwrap().join());
                }
                return;
            }

            if let Some(factor) = input.scale_factor_changed() {
                window.hidpi_factor = factor;
            }
            if let Some(size) = input.window_resized() {
                window.pixels.resize(size.width, size.height);
            }

            //handles updating
            check!(board.try_update());
            if ai_radio.is_some() {check!(ai_radio.as_ref().unwrap().send_board(board.get_board()))} 
            window.window.request_redraw();
        }
    });
}