pub mod pieces;
pub mod strip;
pub use strip::*;

use dynerr::*;
use engine::sprite::Sprite;
use engine::drawing;


use std::fs::OpenOptions;
use std::io::prelude::*;
use std::mem;

///width of board in blocks
const BOARD_WIDTH:          usize           = 10;
///height of board in blocks
const BOARD_HEIGHT:         usize           = 20;
///the left and right padding of board in blocks
const BOARD_PAD:            usize           = 5;
///the screen sprite
const BOARD_SPRITE:         &str            = "board.png";
///the location of the next piece in blocks
const NEXT_PIECE_LOCATION:  (isize, isize)  = (16,1);
///the location of the held piece in blocks
const HELD_PIECE_LOCATION:  (isize, isize)  = (0, 1);
///the color of gameover text
const GAME_OVER_COLOR:      [u8;4]          = [0xFF;4];


///possible piece movements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Move {
    Down,
    Left,
    Right,
}
pub type BoardData = Vec<Vec<Option<pieces::PieceType>>>;
///the board object                                         SHOULD SPLIT UP INTO SEPARATE STRUCTS THAT THE BOARD CAN INTERACT WITH. LIKE "BoardPieces" AND "BoardState"
#[derive(Clone)]
pub struct Board {
    piece:  pieces::Piece,
    shadow: (isize, isize),
    next_piece: pieces::PieceType,
    held_piece: Option<pieces::Piece>,
    spawn: (isize, isize),
    data:   BoardData,
    piece_index: pieces::PieceIndex,
    backdrop: Sprite,
    pub dimensions: (usize, usize),
    padding: usize,
    pub score: usize,
    highscore: usize,
    cleared: usize,
    frame: usize,
    pub level: usize,
    pub gameover: bool,
}

impl Board {
    ///attempts to create a new standard sized board
    pub fn new_board() -> DynResult<Self> {
        let spawn = (BOARD_WIDTH as isize/2-2, 0);
        let piece_index = pieces::PieceType::gen_piece_index();
        let mut board = Self {
            piece: pieces::Piece::gen_piece(pieces::PieceType::pick_random(), spawn, &piece_index),
            shadow: spawn,
            next_piece: pieces::PieceType::pick_random(),
            held_piece: None,
            spawn,
            piece_index,
            backdrop: Sprite::load(BOARD_SPRITE)?,
            dimensions: (0,0),
            padding: BOARD_PAD*pieces::BLOCK_SIZE,
            data: vec!(vec!(None; BOARD_WIDTH); BOARD_HEIGHT),
            score: 0,
            highscore: Self::get_highscore()?,
            cleared: 0,
            frame: 0,
            level: 0,
            gameover: false,
        };
        board.dimensions = (board.backdrop.width, board.backdrop.height);
        board.update_shadow();
        Ok(board)
    }

    ///gets the score from "highscore"
    fn get_highscore() -> DynResult<usize> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("highscore")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.len() == 0 {Ok(0)}
        else {Ok(contents.parse::<usize>()?)}
    }

    ///attempts to hold the current piece
    pub fn hold_piece(&mut self) -> DynResult<bool> {
        if !self.gameover {
            if self.piece.can_hold {
                self.piece.location = self.spawn;
                self.piece.reset_rotation(&self.piece_index);
                if let Some(held) = self.held_piece.take(){
                    self.held_piece = Some(mem::replace(&mut self.piece, held));
                }
                else {
                    unsafe {
                        #[allow(invalid_value)]
                        let piece = mem::replace(&mut self.piece,  mem::MaybeUninit::<pieces::Piece>::zeroed().assume_init());  //UNSAFE CODE BUT DONT WORRY ITS SAFE
                        self.held_piece = Some(piece);
                        if !self.next_piece() {
                            self.piece = self.held_piece.take().unwrap();
                            self.held_piece = None;
                            return Ok(false);
                        };
                    }
                }
                self.piece.can_hold = false;
                self.update_shadow();
                Ok(true)
            } else {Ok(false)}
        } else {Ok(false)}
    }

    ///moves piece down until it gets set
    pub fn drop_piece(&mut self) -> DynResult<bool> {
        if !self.gameover {
            while self.move_piece(Move::Down) {};
            self.update()?;
            Ok(true)
        } else {Ok(false)}
    }

    pub fn rotate_piece(&mut self) -> bool {
        let rotated = self.piece.get_rotated();
        if !self.check_collision(&rotated, rotated.location) {
            self.piece = rotated;
            self.update_shadow();
            true
        } else {false}
    }

    //attempts to move piece. returns bool for success
    pub fn move_piece(&mut self, direction: Move) -> bool {
        let moved = {
            match direction {
                Move::Down  => self.piece.get_down(),
                Move::Left  => self.piece.get_left(),
                Move::Right => self.piece.get_right(),
            }
        };
        if !self.check_collision(&self.piece, moved) {
            self.piece.location = moved;
            if direction != Move::Down {
                self.update_shadow();
            }
            true
        } else {false}
    }

    ///updates the shadow piece
    fn update_shadow(&mut self) {
        let mut shadow = self.piece.location;
        loop {
            let moved = (shadow.0, shadow.1+1);
            if !self.check_collision(&self.piece, moved) {shadow = moved}
            else {break}
        }
        self.shadow = shadow;
    }

    ///attempts to update. returns true if update occurred
    pub fn try_update(&mut self) -> DynResult<bool> {
        self.frame+=1;
        if !self.gameover
        && self.frame%self.get_speed() == 0 {
            self.update()?;
            Ok(true)
        } else {Ok(false)}
    }

    ///gets the current frame delay based on level
    fn get_speed(&self) -> usize {
        match self.level {
            0       =>  48,
            1       =>  43,
            2       =>  38,
            3       =>  33,
            4       =>  28,
            5       =>  23,
            6       =>  18,
            7       =>  13,
            8       =>  8,
            9       =>  6,
            10..=12 =>  5,
            13..=15 =>  4,
            16..=18 =>  3,
            19..=28 =>  2,
            _ =>        1
        }
    }

    /// does game updates
    fn update(&mut self) -> DynResult<()> {
        if !self.move_piece(Move::Down) {
            self.set_piece();
            let cleared = self.update_rows();
            self.update_progress(cleared)?;
            if self.data[0].iter().any(|b| b.is_some())
            || !self.next_piece() {
                self.gameover = true;
            }
        }
        Ok(())
    }

    ///consumes current piece and attempts to set piece
    fn set_piece(&mut self) {
        for row in 0..self.piece.data.len() {
            for block in 0..self.piece.data[row].len() {
                if self.piece.data[row][block] {
                    if let Some(y) = self.data.get_mut((self.piece.location.1+row as isize) as usize) {                //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                        if let Some(x) = y.get_mut((self.piece.location.0+block as isize) as usize) {                  //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                            *x = Some(self.piece.type_);
                        }
                    }
                }
            }
        }
    }

    ///checks for filled rows and removes them
    fn update_rows(&mut self) -> Vec<usize> {
        let mut cleared = Vec::new();
        for row in 0..self.data.len() {
            //if row doesnt have any empty blocks then remove
            if self.data[row].iter().all(|b| b.is_some()) {
                self.data.remove(row);
                self.data.insert(0, vec!(None;self.data[0].len()));
                cleared.push(row);
            }
        }
        cleared
    }

    ///updates score on board and in file
    fn update_progress(&mut self, cleared: Vec<usize>) -> DynResult<()> {
        self.cleared += cleared.len();
        self.level = self.cleared/10;
        let modifier = match cleared.len() {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 3600
        };
        self.score += cleared.iter().map(|row|modifier*(BOARD_HEIGHT-row+1)).sum::<usize>();
        if self.score > self.highscore {
            self.highscore = self.score;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open("highscore")?;
            file.write(format!("{}",self.highscore).as_bytes())?;
        }
        Ok(())
    }

    ///attempts to spawn next piece. returns true on success
    fn next_piece(&mut self) -> bool {
        let next_piece = pieces::Piece::gen_piece(self.next_piece, self.spawn, &self.piece_index);
        if !self.check_collision(&next_piece, self.spawn) {
            loop {
                let attempt_next_piece = pieces::PieceType::pick_random();
                if attempt_next_piece != self.next_piece {
                    self.piece = next_piece;
                    self.next_piece = attempt_next_piece;
                    break
                }
            }
            self.update_shadow();
            true
        } else {false}
    }

    ///takes a piece and checks its collision on the board
    fn check_collision(&self, piece: &pieces::Piece, location: (isize, isize)) -> bool {
        for row in 0..piece.data.len() {
            for block in 0..piece.data[row].len() {
                if piece.data[row][block] {
                    if let Some(y) = self.data.get((location.1+row as isize) as usize) {          //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                        if let Some(x) = y.get((location.0+block as isize) as usize) {            //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                            if x.is_some() {return true}
                        } else {return true}
                    } else {return true}
                }
            }
        }
        false
    }

    ///resets board
    pub fn reset(&mut self) -> DynResult<()> {
        *self = Self::new_board()?;
        Ok(())
    }

    ///draws screen during game play
    pub fn draw(&self, screen: &mut engine::drawing::Screen){
        screen.wipe();
        screen.draw_sprite(&self.backdrop, (0,0));
        //draw set blocks
        for row in 0..self.data.len() {
            for block in 0..self.data[row].len() {
                if let Some(type_) = &self.data[row][block] {
                    let sprite = &self.piece_index.get(&type_).unwrap().0;
                    screen.draw_sprite(sprite, (((block*sprite.width)+self.padding) as isize, (row*sprite.height) as isize))
                }
            }
        }

        for row in 0..self.piece.data.len() {
            for block in 0..self.piece.data[row].len() {
                if self.piece.data[row][block] {
                    let sprite = &self.piece_index.get(&pieces::PieceType::Shadow).unwrap().0;
                    screen.draw_sprite(
                        sprite,
                        (
                            self.shadow.0*sprite.width as isize + ((block*sprite.width)+self.padding) as isize,
                            self.shadow.1*sprite.height as isize + (row*sprite.height) as isize
                        )
                    )
                }
            }
        }

        let next_piece = self.piece_index.get(&self.next_piece).unwrap();
        for row in 0..next_piece.1.len() {
            for block in 0..next_piece.1[row].len() {
                if next_piece.1[row][block] {
                    screen.draw_sprite(
                        &next_piece.0,
                        (
                            NEXT_PIECE_LOCATION.0*next_piece.0.width as isize + (block*next_piece.0.width) as isize,
                            NEXT_PIECE_LOCATION.1*next_piece.0.height as isize + (row*next_piece.0.height) as isize
                        )
                    )
                }
            }
        }

        let mut draw_piece = |piece: &pieces::Piece, location: (isize, isize), padding: usize| {
            for row in 0..piece.data.len() {
                for block in 0..piece.data[row].len() {
                    if piece.data[row][block] {
                        let sprite = &self.piece_index.get(&piece.type_).unwrap().0;
                        screen.draw_sprite(
                            sprite,
                            (
                                location.0*sprite.width as isize + ((block*sprite.width)+padding) as isize,
                                location.1*sprite.height as isize + (row*sprite.height) as isize
                            )
                        )
                    }
                }
            }
        };
        draw_piece(&self.piece, self.piece.location, self.padding);
        if let Some(held) = &self.held_piece {
            draw_piece(held, HELD_PIECE_LOCATION, 0);
        }

        screen.draw_text((9,191), &format!("{}",self.highscore), 32.0, &[255;4], drawing::DEBUG_FONT);
        screen.draw_text((9,254), &format!("{}",self.score), 32.0, &[255;4], drawing::DEBUG_FONT);
        screen.draw_text((83,287), &format!("{:02}",self.level), 32.0, &[255;4], drawing::DEBUG_FONT);

        if self.gameover {
            screen.draw_text((195 ,40), "GAME OVER", 64.0, &GAME_OVER_COLOR, drawing::DEBUG_FONT);
            let message = format!("SCORE: {}",self.score);
            screen.draw_text((225,115), &message, 32.0, &GAME_OVER_COLOR, drawing::DEBUG_FONT);
            screen.draw_text((215,200), "SPACE TO RESTART", 32.0, &GAME_OVER_COLOR, drawing::DEBUG_FONT);
        }
    }

    pub fn get_board(&self) -> StrippedBoard {
        StrippedBoard::get(&self)
    }
}













pub mod tests {
    use dynerr::*;
    pub fn get_highscore() -> DynResult<usize> {
        super::Board::get_highscore()
    }

    pub fn update_shadow(board: &mut super::Board) {
        board.update_shadow();
    }

    pub fn get_speed(board: &super::Board) -> usize {
        board.get_speed()
    }

    pub fn update(board: &mut super::Board) -> DynResult<()> {
        board.update()
    }

    pub fn set_piece(board: &mut super::Board) {
        board.set_piece()
    }

    pub fn update_rows(board: &mut super::Board) -> Vec<usize> {
        board.update_rows()
    }

    pub fn update_progress(board: &mut super::Board, cleared: Vec<usize>) -> DynResult<()> {
        board.update_progress(cleared)
    }

    pub fn next_piece(board: &mut super::Board) -> bool {
        board.next_piece()
    }

    pub fn check_collision(board: &mut super::Board, piece: &super::pieces::Piece, location: (isize, isize)) -> bool {
        board.check_collision(piece, location)
    }

    pub fn assist_get_piece(board: &super::Board) -> super::pieces::Piece {
        board.piece.clone()
    }

    pub fn assist_get_board_data(board: &super::Board) -> super::BoardData {
        board.data.clone()
    }
}