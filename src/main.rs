mod tetris;

use dynerr::*;
use engine::{drawing, game};

const SCREEN_HEIGHT: usize = 1080/2;
const SCREEN_WIDTH: usize = 1920/3;
const TARGET_FPS: u64 = 60;
const GAME_TITLE: &str = "Tetris";

fn main() {
    let mut board = tetris::Board::new_standard().unwrap_or_else(|e| logged_panic!(e));

    let mut screen = drawing::Screen::new(                                                              //create blank screen buffer
        tetris::STANDARD_WIDTH*tetris::BLOCK_SIZE,
        tetris::STANDARD_HEIGHT*tetris::BLOCK_SIZE
    );

    let mut fpslock = game::FpsLock::create_lock(TARGET_FPS);                                           //create fpslock obj

    let event_loop = game::EventLoop::new();                                                            //create event loop obj
    let mut input = game::WinitInputHelper::new();                                                      //create input helper obj
    let mut window = game::Window::init(GAME_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT, &event_loop);          //create window, and pixels buffer


    event_loop.run(move |event, _, control_flow| {                                                      //start game loop
        fpslock.start_frame();                                                                          //set start of frame time
        if let game::Event::RedrawRequested(_) = event {                                                //if redraw requested

            //MODIFY SCREEN HERE
            screen.wipe();                                                                              //EXAMPLE draw black to entire screen
            board.draw(&mut screen);
            let displayfps = format!("FPS: {}",fpslock.get_fps());                                      //EXAMPLE get fps
            screen.draw_text((20,40), &displayfps, 32.0, &[255;4], drawing::DEBUG_FONT);                 //EXAMPLE draw fps to screen as white

            screen.flatten(window.pixels.get_frame());                                                  //flatten screen to 1d Vec<[u8;4]>
            window.pixels.render().unwrap();                                                            //render

            fpslock.end_frame();
        }

        if input.update(event) {                                                                        //handle input events on loop? not just on event

            //GET GAME INPUT HERE
            //info on keys at https://docs.rs/winit/0.5.2/winit/enum.VirtualKeyCode.html
            //info on events at https://docs.rs/winit_input_helper/0.7.0/winit_input_helper/struct.WinitInputHelper.html
            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                        //if esc pressed
                *control_flow = game::ControlFlow::Exit;                                                //exit
                return;
            }

            if input.key_held(game::VirtualKeyCode::A) {board.piece_left();}                                  //EXAMPLE
            if input.key_held(game::VirtualKeyCode::S) {board.piece_down();}                                  //EXAMPLE
            if input.key_held(game::VirtualKeyCode::D) {board.piece_right();}                                  //EXAMPLE
            if input.key_held(game::VirtualKeyCode::R) {board.try_rotate();}                                  //EXAMPLE
            if input.key_pressed(game::VirtualKeyCode::Space){board.drop_piece()}                         //EXAMPLE
            if input.key_pressed(game::VirtualKeyCode::F3) {println!("F3")}                             //EXAMPLE

            if let Some(factor) = input.scale_factor_changed() {                                        //if window dimensions changed
                window.hidpi_factor = factor;                                                           //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                //if window resized
                window.pixels.resize(size.width, size.height);                                          //resize pixel aspect ratio
            }

            //DO WORLD UPDATES HERE
            //NO EXAMPLES
            board.update();
            window.window.request_redraw();                                                             //request frame redraw
        }
    });
}