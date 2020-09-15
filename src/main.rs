#![windows_subsystem = "windows"]
mod tetris;

use dynerr::*;
use engine::{drawing, game};

use std::time::Duration;
use std::thread::sleep;


const TARGET_FPS: u64 = 60;
const START_SPEED: usize = 20;
const GAME_TITLE: &str = "Tetris";
const GAME_OVER_PAUSE: usize= 5;



struct UpdateManager {
    frame: usize,
    speed: usize,
    start_speed: usize,
}
impl UpdateManager {
    fn new(speed: usize) -> Self {
        Self {
            frame: 0,
            speed,
            start_speed: speed
        }
    }

    fn get_speed(&self) -> usize {
        self.start_speed - self.speed
    }

    fn reset(&mut self) {
        self.frame = 0;
        self.speed = self.start_speed;
    }

    fn should_update(&mut self) -> bool {
        self.frame+=1;
        if self.frame % self.speed == 0 {
            if self.frame >= 2000 {
                self.speed = self.speed.checked_sub(1).unwrap_or(0);
                self.frame = 0;
            }
            true
        } else {false}
    }
}

fn main() {
    let mut board = check!(tetris::Board::new_standard());
    let mut update_manager = UpdateManager::new(START_SPEED);

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
            screen.draw_text((10,20), &format!("Speed: {}",update_manager.get_speed()), 16.0, &[255;4], drawing::DEBUG_FONT);
            screen.draw_text((10,40), &format!("Score: {}",board.score), 16.0, &[255;4], drawing::DEBUG_FONT);
            screen.draw_text((10,60), &format!("Highscore: {}",board.highscore), 16.0, &[255;4], drawing::DEBUG_FONT);


            screen.flatten(window.pixels.get_frame());                                                  //flatten screen to 1d Vec<[u8;4]>
            window.pixels.render().unwrap();                                                            //render

            fpslock.end_frame();
        }

        if input.update(event) {                                                                        //handle input events on loop? not just on event

            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                        //if esc pressed
                *control_flow = game::ControlFlow::Exit;                                                //exit
                return;
            }

            if input.key_pressed(game::VirtualKeyCode::A)    {board.piece_left();}
            if input.key_pressed(game::VirtualKeyCode::S)    {board.piece_down();}
            if input.key_pressed(game::VirtualKeyCode::D)    {board.piece_right();}
            if input.key_pressed(game::VirtualKeyCode::R)    {board.piece_rotate();}
            if input.key_pressed(game::VirtualKeyCode::Space){board.piece_drop();}
            //if input.key_pressed(game::VirtualKeyCode::F3) {println!("F3")}


            if let Some(factor) = input.scale_factor_changed() {                                        //if window dimensions changed
                window.hidpi_factor = factor;                                                           //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                //if window resized
                window.pixels.resize(size.width, size.height);                                          //resize pixel aspect ratio
            }

            //handles updating
            if update_manager.should_update() {
                if check!(board.update()) {
                    for i in (1..=GAME_OVER_PAUSE).rev() {
                        board.draw_gameover(&mut screen, update_manager.get_speed(), i);
                        screen.flatten(window.pixels.get_frame());
                        check!(window.pixels.render());
                        sleep(Duration::from_secs(1));
                    }
                    check!(board.reset());
                    update_manager.reset();
                }
            }
            window.window.request_redraw();                                                             //request frame redraw
        }
    });
}