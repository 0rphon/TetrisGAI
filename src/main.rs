//#![windows_subsystem = "windows"]         UNCOMMENT FOR RELEASE
mod tetris;

use dynerr::*;
use engine::{drawing, game};

use std::time::Duration;
use std::thread::sleep;


///the target fps
const TARGET_FPS: u64 = 60;
///start out doing updates every x frames
const START_SPEED: usize = 30;
///how many updates til you increase speed level
const SPEED_LEVEL_DURATION: usize = 2500;
const GAME_TITLE: &str = "Tetris";
const GAMEOVER_SECS: f32 = 5.0;
const GAMEOVER_TICK_LEN: f32 = 0.25;



struct UpdateManager {
    frame: usize,
    speed: usize,
    start_speed: usize,
    gameover: bool,
    gameover_ticks: usize,
    gameover_prog:  usize,
}
impl UpdateManager {
    fn new() -> Self {
        Self {
            frame: 0,
            speed: START_SPEED,
            start_speed: START_SPEED,
            gameover: false,
            gameover_ticks: (GAMEOVER_SECS/GAMEOVER_TICK_LEN) as usize,
            gameover_prog: 0
        }
    }

    ///gets current speed
    fn get_speed(&self) -> usize {
        self.start_speed - self.speed
    }

    ///checks if should update then updates
    fn try_update(&mut self, board: &mut tetris::Board) -> DynResult<()> {
        if !self.gameover {
            self.frame+=1;
            if self.frame % self.speed == 0 {
                if self.frame >= SPEED_LEVEL_DURATION {
                    self.speed = self.speed.checked_sub(1).unwrap_or(0);
                    self.frame = 0;
                }
                if board.update()? {
                    self.gameover = true;
                }
            }
        }
        Ok(())
    }

    ///checks if gameover is currently happening
    fn is_gameover(&mut self, board: &mut tetris::Board) -> DynResult<bool> {
        if self.gameover {
            if self.gameover_prog < self.gameover_ticks {
                self.gameover_prog+=1;
                Ok(true)
            } else {
                board.reset()?;
                *self = Self::new();        
                Ok(false)
            }
        } else {Ok(false)}
    }

    ///gets seconds til end of gameover
    fn sec_til_restart(&self) -> usize {
        (GAMEOVER_SECS-(GAMEOVER_TICK_LEN*self.gameover_prog as f32)) as usize+1
    }
}

fn main() {
    let mut board = check!(tetris::Board::new_board());
    let mut update_manager = UpdateManager::new();

    let mut screen = drawing::Screen::new(                                                              
        board.width,
        board.height
    );

    let mut fpslock = game::FpsLock::create_lock(TARGET_FPS);                                           

    let event_loop = game::EventLoop::new();                                                            
    let mut input = game::WinitInputHelper::new();                                                      
    let mut window = game::Window::init(                                                                
        GAME_TITLE,
        board.width,
        board.height,
        &event_loop
    );

    event_loop.run(move |event, _, control_flow| {                                                      
        fpslock.start_frame();                                                                          
        if let game::Event::RedrawRequested(_) = event {                                                

            screen.wipe();
            if check!(update_manager.is_gameover(&mut board)) {
                board.draw_gameover(&mut screen, update_manager.get_speed(), update_manager.sec_til_restart());
                sleep(Duration::from_secs_f32(GAMEOVER_TICK_LEN));
            } else {
                board.draw(&mut screen, update_manager.get_speed());
            }
            screen.flatten(window.pixels.get_frame());                                                  
            window.pixels.render().unwrap();                                                            

            fpslock.end_frame();
        }
        
        //attempt at getting gamepad input. didnt work
        //match event {
        //    game::Event::DeviceEvent{device_id: id, event: _} => {println!("{:X?}",id)},
        //    _ => {}
        //};

        if input.update(&event) {                                                                       

            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                        
                *control_flow = game::ControlFlow::Exit;                                                
                return;
            }

            if !update_manager.gameover {
                if input.key_pressed(game::VirtualKeyCode::A)
                || input.key_pressed(game::VirtualKeyCode::Left)    {board.piece_left();}

                if input.key_pressed(game::VirtualKeyCode::S)
                || input.key_pressed(game::VirtualKeyCode::Down)    {board.piece_down();}

                if input.key_pressed(game::VirtualKeyCode::D)
                || input.key_pressed(game::VirtualKeyCode::Right)   {board.piece_right();}

                if input.key_pressed(game::VirtualKeyCode::R)
                || input.key_pressed(game::VirtualKeyCode::X)    
                || input.key_pressed(game::VirtualKeyCode::Up)      {board.piece_rotate();}


                if input.key_pressed(game::VirtualKeyCode::F)    
                ||input.key_pressed(game::VirtualKeyCode::C)        {check!(board.piece_hold());}

                if input.key_pressed(game::VirtualKeyCode::Space)   {board.piece_drop();}
            }

            if let Some(factor) = input.scale_factor_changed() {                                        
                window.hidpi_factor = factor;                                                           
            }
            if let Some(size) = input.window_resized() {                                                
                window.pixels.resize(size.width, size.height);                                          
            }

            //handles updating
            check!(update_manager.try_update(&mut board));
            window.window.request_redraw();                                                             
        }
    });
}