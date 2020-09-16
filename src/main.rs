#![windows_subsystem = "windows"]         //UNCOMMENT FOR RELEASE
mod tetris;

use dynerr::*;
use engine::{drawing, game};

///the target fps
const TARGET_FPS: u64 = 60;
const GAME_TITLE: &str = "Tetris";


fn main() {
    let mut board = check!(tetris::Board::new_board());

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
            board.draw(&mut screen);
            //screen.draw_text((0,0), fpslock.get_fps(), 16.0, &[0xFF;4], drawing::DEBUG_FONT);
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

            if input.key_pressed(game::VirtualKeyCode::A)
            || input.key_pressed(game::VirtualKeyCode::Left)    
            {board.piece_left();}
            
            if input.key_pressed(game::VirtualKeyCode::S)
            || input.key_pressed(game::VirtualKeyCode::Down)    
            {board.piece_down();}
            
            if input.key_pressed(game::VirtualKeyCode::D)
            || input.key_pressed(game::VirtualKeyCode::Right)   
            {board.piece_right();}
            
            if input.key_pressed(game::VirtualKeyCode::W)
            || input.key_pressed(game::VirtualKeyCode::R)
            || input.key_pressed(game::VirtualKeyCode::X)    
            || input.key_pressed(game::VirtualKeyCode::Up)      
            {board.piece_rotate();}
            
            
            if input.key_pressed(game::VirtualKeyCode::F)    
            ||input.key_pressed(game::VirtualKeyCode::C)        
            {check!(board.piece_hold());}
            
            if input.key_pressed(game::VirtualKeyCode::Space)   
            {check!(board.piece_drop());}
            
            if input.key_pressed(game::VirtualKeyCode::Escape) || input.quit() {                        
                *control_flow = game::ControlFlow::Exit;                                                
                return;
            }

            if input.key_pressed(game::VirtualKeyCode::Return)
            || input.key_pressed(game::VirtualKeyCode::NumpadEnter) 
            || input.key_pressed(game::VirtualKeyCode::Space) && board.gameover 
                {check!(board.reset())}

            if let Some(factor) = input.scale_factor_changed() {                                        
                window.hidpi_factor = factor;                                                           
            }
            if let Some(size) = input.window_resized() {                                                
                window.pixels.resize(size.width, size.height);                                          
            }

            //handles updating
            check!(board.try_update());
            window.window.request_redraw();                                                             
        }
    });
}