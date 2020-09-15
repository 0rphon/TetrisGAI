mod tetris;

use dynerr::*;
use engine::{drawing, game};

const TARGET_FPS: u64 = 10;
const FRAME_PER_UPDATE: u32 = 5;
const GAME_TITLE: &str = "Tetris";

struct UpdateFlags {
    frame: u32,
    target: u32,
    set: bool,
}
impl UpdateFlags {
    fn new(target: u32) -> Self {
        Self {
            frame: 0,
            target,
            set: false,
        }
    }

    fn check(&self) -> bool {
        if self.frame >= self.target || self.set {
            true
        } else {false}
    }
}

fn main() {
    let mut board = check!(tetris::Board::new_standard());
    let mut update_flags = UpdateFlags::new(FRAME_PER_UPDATE);

    let mut screen = drawing::Screen::new(                                                              //create blank screen buffer
        tetris::STANDARD_WIDTH*tetris::BLOCK_SIZE,
        tetris::STANDARD_HEIGHT*tetris::BLOCK_SIZE
    );

    let mut fpslock = game::FpsLock::create_lock(TARGET_FPS);                                           //create fpslock obj

    let event_loop = game::EventLoop::new();                                                            //create event loop obj
    let mut input = game::WinitInputHelper::new();                                                      //create input helper obj
    let mut window = game::Window::init(                                                                //create window, and pixels buffer
        GAME_TITLE,
        tetris::STANDARD_WIDTH*tetris::BLOCK_SIZE,
        tetris::STANDARD_HEIGHT*tetris::BLOCK_SIZE,
        &event_loop
    );

    event_loop.run(move |event, _, control_flow| {                                                      //start game loop
        fpslock.start_frame();                                                                          //set start of frame time
        if let game::Event::RedrawRequested(_) = event {                                                //if redraw requested

            screen.wipe();
            board.draw(&mut screen);
            screen.draw_text((10,20), &format!("Score: {}",board.score), 16.0, &[255;4], drawing::DEBUG_FONT);
            screen.draw_text((10,40), &format!("Highscore: {}",board.highscore), 16.0, &[255;4], drawing::DEBUG_FONT);


            screen.flatten(window.pixels.get_frame());                                                  //flatten screen to 1d Vec<[u8;4]>
            window.pixels.render().unwrap();                                                            //render

            //use std::{thread, time};
            //thread::sleep(time::Duration::from_secs(1));
            fpslock.end_frame();
        }

        if input.update(event) {                                                                        //handle input events on loop? not just on event

            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                        //if esc pressed
                *control_flow = game::ControlFlow::Exit;                                                //exit
                return;
            }

            if input.key_held(game::VirtualKeyCode::A) {board.piece_left();}
            if input.key_held(game::VirtualKeyCode::S) {
                if !board.piece_down() {
                    update_flags.set = true;
                }
            }
            if input.key_held(game::VirtualKeyCode::D) {board.piece_right();}
            if input.key_held(game::VirtualKeyCode::R) {board.try_rotate();}
            if input.key_pressed(game::VirtualKeyCode::Space){
                board.drop_piece();
                update_flags.set = true;
            }
            if input.key_pressed(game::VirtualKeyCode::F3) {println!("F3")}

            if let Some(factor) = input.scale_factor_changed() {                                        //if window dimensions changed
                window.hidpi_factor = factor;                                                           //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                //if window resized
                window.pixels.resize(size.width, size.height);                                          //resize pixel aspect ratio
            }

            //handles updating
            if update_flags.check() {
                check!(board.update());
                update_flags = UpdateFlags::new(FRAME_PER_UPDATE);
            } else {
                update_flags.frame += 1;
            }
            window.window.request_redraw();                                                             //request frame redraw
        }
    });
}