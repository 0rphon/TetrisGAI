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

pub fn run(train: bool, auto_loop: bool, use_best: bool) {
    if train {
        check!(train::train());
        return
    }

    //UNTRAINED                        3   : 0.500   : 0.500   : 0.250   : 0.750   : 0.000   : 0.500   : 3.500   : 2   : 0.750
    //         ???  |    ?? |    ??? | 2   : 0.919   : 0.006   : 0.572   : 0.049   : 0.143   : 0.012   : 0.995   : 0   : 0.392
    //      392860  |    8  |    227 | 2   : 0.939   : 0.004   : 0.240   : 0.002   : 0.102   : 0.024   : 0.920   : 0   : 0.200
    //      693944  |    6  |    196 | 4   : 0.79000 : 0.00100 : 0.14900 : 0.05000 : 0.09200 : 0.01900 : 0.63200 : 1   : 0.06600
    //      1189945 |   18  |    485 | 4   : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0   : 0.06300
    //      1145311 |   21  |    556 | 4.0 : 0.91910 : 0.00000 : 0.56862 : 0.09960 : 0.32070 : 0.06357 : 1.00564 : 1.0 : 0.38466
    //208 | 1170614 |   16  |    443 | 0.0 : -0.61509 : 0.00227 : 1.49888 : 0.89981 : -1.63555 : -0.83318 : 1.94491 : 0.0 : 0.90794
    // 75 | 1158106 |   17  |    475 | 4.0 : 2.81089 : 0.00000 : 1.44516 : 0.00000 : 1.38000 : 0.12947 : 2.62338 : 0.0 : 0.97880
    //202 | 1037750 |    8  |    252 | 4.0 : 0.78193 : 0.00000 : 0.51338 : 0.09517 : 0.00000 : 0.04101 : 1.00000 : 0.0 : 0.22724
    //251 | 2393169 |   37  |    958 | 4.0 : 0.91413 : 0.00000 : 0.66610 : 0.01078 : 0.22913 : 0.05655 : 0.78533 : 0.0 : 0.27396        9.36M lvl 157 i think thats good enough tbh. it got 60k per level...my record is like 32k/lvl
    // 38 | 2030640 |   40  |   1051 | 4.0 : 0.83434 : 0.00000 : 0.98846 : 0.04482 : 0.09175 : 0.00000 : 0.89960 : 0.0 : 0.34672        7.71M lvl 166
    let parameters = {
        if use_best {
            match check!(train::BestResult::get_best()) {
                Some(params) => params,
                None         => logged_panic!("Couldnt find best.log! Have you trained the ai at all?"),
            }
        } else {
            ai::AiParameters {
                min_lines_to_clear:             4.0,
                lines_cleared_importance:       0.83434,
                points_scored_importance:       0.00000,
                piece_depth_importance:         0.98846,
                max_height_importance:          0.04482,
                avg_height_importance:          0.09175,
                height_variation_importance:    0.00000,
                current_holes_importance:       0.89960,
                max_pillar_height:              0.0,
                current_pillars_importance:     0.34672,
            }
        }
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
                        None => Some(ai::start(parameters.clone(), false)),     //bool to turn on debug logging
                    }
                }
            }
            if ai_radio.is_some() {
                if let Some(ai_input) = check!(ai_radio.as_ref().unwrap().get_input()) {
                    match ai_input {
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
            if ai_radio.is_some() {check!(ai_radio.as_ref().unwrap().send_board(board.get_board()))}
            else {check!(board.try_update());}
            window.window.request_redraw();
        }
    });
}