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

pub fn run(train: bool, dry: bool) {
    if train {
        check!(train::train(dry));
        return
    }

    let parameters = ai::AiParameters {
        points_scored_importance:       0.50,
        piece_depth_importance:         0.25,
        max_height_importance:          0.75,
        avg_height_importance:          0.0,
        height_variation_importance:    0.5,
        current_holes_importance:       3.5,
        max_pillar_height:              2,
        current_pillars_importance:     0.75,
    };
    let mut board = check!(Board::new_board());
    let mut ai_radio = None;

    let mut screen = engine::drawing::Screen::new(
        board.dimensions.0,
        board.dimensions.1
    );
    let mut fpslock = engine::game::FpsLock::create_lock(TARGET_FPS);
    let event_loop = engine::game::EventLoop::new();
    let mut input = engine::game::WinitInputHelper::new();
    let mut window = engine::game::Window::init(
        GAME_TITLE,
        board.dimensions.0,
        board.dimensions.1,
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
                        None => Some(ai::start(parameters.clone(), true)),
                    }
                }
            }
            if ai_radio.is_some() {
                if let Some(ai_input) = check!(ai_radio.as_ref().unwrap().get_input()) {
                    match ai_input {
                        //ai::Move::Down      => {board.move_piece(Move::Down);},
                        ai::Move::Left      => {board.move_piece(Move::Left);},
                        ai::Move::Right     => {board.move_piece(Move::Right);},
                        ai::Move::Rotate    => {board.move_piece(Move::Rotate);}
                        ai::Move::Drop      => {board.move_piece(Move::Drop);},
                        ai::Move::Hold      => {check!(board.piece_hold());},
                        ai::Move::Restart   => {},//check!(board.reset()),
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
                {board.move_piece(Move::Rotate);}


                if input.key_pressed(engine::game::VirtualKeyCode::F)
                ||input.key_pressed(engine::game::VirtualKeyCode::C)
                {check!(board.piece_hold());}

                if input.key_pressed(engine::game::VirtualKeyCode::Space)
                {check!(board.piece_drop());}
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
            if check!(board.try_update()) && ai_radio.is_some(){
                check!(ai_radio.as_ref().unwrap().send_board(board.get_board()));
            }
            window.window.request_redraw();
        }
    });
}