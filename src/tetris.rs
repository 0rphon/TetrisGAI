use dynerr::*;
use engine::sprite::Sprite;
use engine::drawing;

use rand::Rng;
use std::{fmt, error};
use std::fs::OpenOptions;
use std::io::prelude::*;

///the size of each block. used to calc grid
const BLOCK_SIZE:           usize           = 32;
///the thickness of piece border in pixels
const BORDER_SIZE:          usize           = 2;
///the color of piece borders
const BORDER_COLOR:         [u8;4]          = [0x00, 0x00, 0x00, 0xFF];
///the color of shadow
const SHADOW_COLOR:         [u8;4]          = [0x00;4];
///the color of the shadows border
const SHADOW_BORDER_COLOR:  [u8;4]          = [0xDC, 0xDC, 0xDC, 0xFF];     //[0X00;4] TO USE THE PIECE COLOR FOR SHADOW BORDER

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



const I_COLOR: [u8;4] = [0x00, 0xFF, 0xFF, 0xFF];
const I_DATA: [&[bool;4];4] = [
    &[false, false, false, false],
    &[true , true , true , true ],
    &[false, false, false, false],
    &[false, false, false, false],
];
const O_COLOR: [u8;4] = [0xFF, 0xFF, 0x00, 0xFF];
const O_DATA: [&[bool;2];2] = [
    &[true , true],
    &[true , true],
];
const T_COLOR: [u8;4] = [0x80, 0x00, 0x80, 0xFF];
const T_DATA: [&[bool;3];3] = [
    &[false, true , false],
    &[true , true , true ],
    &[false, false, false],
];
const S_COLOR: [u8;4] = [0x00, 0x80, 0x00, 0xFF];
const S_DATA: [&[bool;3];3] = [
    &[false, true , true ],
    &[true , true , false],
    &[false, false, false],
];
const Z_COLOR: [u8;4] = [0xFF, 0x00, 0x00, 0xFF];
const Z_DATA: [&[bool;3];3] = [
    &[true , true , false],
    &[false, true , true ],
    &[false, false, false],
];
const J_COLOR: [u8;4] = [0x00, 0x00, 0xFF, 0xFF];
const J_DATA: [&[bool;3];3] = [
    &[true , false, false],
    &[true , true , true ],
    &[false, false, false],
];
const L_COLOR: [u8;4] = [0xFF, 0xA5, 0x00, 0xFF];
const L_DATA: [&[bool;3];3] = [
    &[false, false, true ],
    &[true , true , true ],
    &[false, false, false],
];

///converts [&[T; Size]; Size] to Vec<Vec<T>>
macro_rules! fit {
    ($x:expr) => {
        $x.iter().map(|y| y.to_vec()).collect::<Vec<_>>()
    };
}

///a custom error type
#[derive(Debug)]
enum TetrisError {
    SpawnError((isize, isize)),
    GenerationError(u32),
}
//impl display formatting for error
impl fmt::Display for TetrisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TetrisError::SpawnError((x,y))      => write!(f, "TetrisError::SpawnError: failed to spawn piece at x:{} y:{}", x, y),
            TetrisError::GenerationError(i)     => write!(f, "PieceError::GenerationError: random number generator returned invalid value: {}", i),
        }
    }
}
//impl error conversion for error
impl error::Error for TetrisError {}











///blocks in piece
type PieceData = Vec<Vec<Option<Sprite>>>;
///piece types
#[derive(Clone, PartialEq)]
enum PieceType {I, O, T, S, Z, J, L}
///the piece object
#[derive(Clone)]
struct Piece {
    type_: PieceType,
    location: (isize, isize),
    data: PieceData,
    can_hold: bool,
}
impl Piece {
    ///generates a colored block with a border
    fn gen_block(color: [u8;4]) -> Sprite {
        let mut block = vec!(vec!(color; BLOCK_SIZE); BLOCK_SIZE);
        for row_i in 0..BLOCK_SIZE {
            for pixel_i in 0..BLOCK_SIZE {
                if (0..BORDER_SIZE).contains(&row_i)
                || (0..BORDER_SIZE).contains(&pixel_i)
                || (BLOCK_SIZE-BORDER_SIZE..BLOCK_SIZE).contains(&row_i)
                || (BLOCK_SIZE-BORDER_SIZE..BLOCK_SIZE).contains(&pixel_i) {
                    block[row_i][pixel_i] = BORDER_COLOR;
                }
            }
        }
        Sprite::add(BLOCK_SIZE, BLOCK_SIZE, block)
    }

    ///generates a piece's block data
    fn gen_piece(shape: Vec<Vec<bool>>, color: [u8;4]) -> PieceData {
        shape.iter().map(|row|
            row.iter().map(|block|
                if *block {
                    Some(Self::gen_block(color))
                } else {
                    None
                }
            ).collect()
        ).collect()
    }

    ///generates an I piece
    fn new_i(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::I,
            location,
            data: Self::gen_piece(fit!(I_DATA), I_COLOR),
            can_hold: true,
        }
    }

    ///generates an O piece
    fn new_o(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::O,
            location,
            data: Self::gen_piece(fit!(O_DATA), O_COLOR),
            can_hold: true,
        }
    }

    ///generates a T piece
    fn new_t(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::T,
            location,
            data: Self::gen_piece(fit!(T_DATA), T_COLOR),
            can_hold: true,
        }
    }

    ///generates an S piece
    fn new_s(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::S,
            location,
            data: Self::gen_piece(fit!(S_DATA), S_COLOR),
            can_hold: true,
        }
    }

    ///generates a Z piece
    fn new_z(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::Z,
            location,
            data: Self::gen_piece(fit!(Z_DATA), Z_COLOR),
            can_hold: true,
        }
    }

    ///generates a J piece
    fn new_j(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::J,
            location,
            data: Self::gen_piece(fit!(J_DATA), J_COLOR),
            can_hold: true,
        }
    }

    ///generates an L piece
    fn new_l(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::L,
            location,
            data: Self::gen_piece(fit!(L_DATA), L_COLOR),
            can_hold: true,
        }
    }

    ///attempts to generate a random piece
    fn gen_random(location: (isize, isize)) -> DynResult<Self> {
        match rand::thread_rng().gen_range(0, 7) {
            0 => {Ok(Self::new_i(location))},
            1 => {Ok(Self::new_j(location))},
            2 => {Ok(Self::new_l(location))},
            3 => {Ok(Self::new_o(location))},
            4 => {Ok(Self::new_s(location))},
            5 => {Ok(Self::new_t(location))},
            6 => {Ok(Self::new_z(location))},
            i => dynerr!(TetrisError::GenerationError(i))
        }
    }

    ///gets the shadow of a piece
    fn get_shadow(&self) -> Piece {
        let mut shadow = self.clone();
        for row in 0..shadow.data.len() {
            for block in 0..shadow.data[row].len() {
                if let Some(sprite) = shadow.data[row][block].clone() {
                    shadow.data[row][block].as_mut().unwrap().img = sprite.img.iter().map(|y|
                        y.iter().map(|x|
                            if *x != BORDER_COLOR {SHADOW_COLOR}
                            else {
                                if SHADOW_BORDER_COLOR == [0x00;4] {
                                    match self.type_ {
                                    PieceType::I => {I_COLOR}
                                    PieceType::O => {O_COLOR}
                                    PieceType::T => {T_COLOR}
                                    PieceType::S => {S_COLOR}
                                    PieceType::Z => {Z_COLOR}
                                    PieceType::J => {J_COLOR}
                                    PieceType::L => {L_COLOR}
                                    }
                                } else {SHADOW_BORDER_COLOR}
                                
                            }
                        ).collect()
                    ).collect();
                }
            }
        }
        shadow
    }

    ///gets a rotated version of the piece
    fn get_rotated(&self) -> Piece {
        let height = self.data.len();
        let width = self.data[0].len();                                         //UNCHECKED INDEX
        let mut r = self.clone();
        for row in 0..height {
            for block in 0..width {
                if let Some(sprite) = &self.data[row][block] {
                    r.data[block][width-row-1] = Some(sprite.clone());
                } else {
                    r.data[block][width-row-1] = None;
                }
            }
        }
        r
    }

    ///gets a moved version of the piece
    fn get_up(&self) -> Self {
        let mut moved = self.clone();
        if let Some(y) = moved.location.1.checked_sub(1) {
            moved.location.1 = y;
        }
        moved
    }

    ///gets a moved version of the piece
    fn get_down(&self) -> Self {
        let mut moved = self.clone();
        if let Some(y) = moved.location.1.checked_add(1) {
            moved.location.1 = y;
        }
        moved
    }

    ///gets a moved version of the piece
    fn get_left(&self) -> Self {
        let mut moved = self.clone();
        if let Some(x) = moved.location.0.checked_sub(1) {
            moved.location.0 = x;
        }
        moved
    }

    ///gets a moved version of the piece
    fn get_right(&self) -> Self {
        let mut moved = self.clone();
        if let Some(x) = moved.location.0.checked_add(1) {
            moved.location.0 = x;
        }
        moved
    }
}












///possible piece movements
enum Move {
    Up,
    Down,
    Left,
    Right,
    Rotate,
    Drop,
}

///the board object
#[derive(Clone)]
pub struct Board {
    piece:  Piece,
    shadow: Piece,
    next_piece: Piece,
    held_piece: Option<Piece>,
    spawn: (isize, isize),
    data:   Vec<Vec<Option<Sprite>>>,
    backdrop: Sprite,
    pub width: usize,
    pub height: usize,
    padding: usize,
    pub score: usize,
    pub highscore: usize,
}

impl Board {
    ///attempts to create a new standard sized board
    pub fn new_board() -> DynResult<Self> {
        let spawn = (BOARD_WIDTH as isize/2-2, 0);
        let mut board = Self {
            piece: Piece::gen_random(spawn)?,
            shadow: Piece::gen_random(spawn)?,
            next_piece: Piece::gen_random(spawn)?,
            held_piece: None,
            spawn,
            backdrop: Sprite::load(BOARD_SPRITE)?,
            width: 0,
            height: 0,
            padding: BOARD_PAD*BLOCK_SIZE,
            data: vec!(vec!(None; BOARD_WIDTH); BOARD_HEIGHT),
            score: 0,
            highscore: Self::get_highscore()?
        };
        board.width = board.backdrop.width;
        board.height = board.backdrop.height;
        board.update_shadow();
        Ok(board)
    }

    ///attempts to hold the current piece
    pub fn piece_hold(&mut self) -> DynResult<bool> {
        if self.piece.can_hold {
            self.piece.location = self.spawn;
            if let Some(held) = self.held_piece.clone() {
                self.held_piece = Some(self.piece.clone());
                self.piece = held;
            }
            else {
                self.held_piece = Some(self.piece.clone());
                self.next_piece()?;
            }
            self.piece.can_hold = false;
            self.update_shadow();
            Ok(true)
        } else {Ok(false)}
    }

    ///attempts to rotate the piece
    pub fn piece_rotate(&mut self) -> bool {
        self.move_piece(Move::Rotate)
    }

    ///attempts to move piece up
    #[allow(unused)]
    pub fn piece_up(&mut self) -> bool {
        self.move_piece(Move::Up)
    }

    /// attempts to move piece down
    pub fn piece_down(&mut self) -> bool {
        self.move_piece(Move::Down)
    }

    ///attempts to move piece left
    pub fn piece_left(&mut self) -> bool {
        self.move_piece(Move::Left)
    }

    ///attempts to move piece right
    pub fn piece_right(&mut self) -> bool {
        self.move_piece(Move::Right)
    }

    ///moves piece down until it gets set
    pub fn piece_drop(&mut self) -> bool {
        self.move_piece(Move::Drop)
    }

    //attempts to move piece. returns bool for success
    fn move_piece(&mut self, direction: Move) -> bool {
        let moved = {
            match direction {
                Move::Up    => self.piece.get_up(),
                Move::Down  => self.piece.get_down(),
                Move::Left  => self.piece.get_left(),
                Move::Right => self.piece.get_right(),
                Move::Rotate=> self.piece.get_rotated(),
                Move::Drop  => {
                    while self.move_piece(Move::Down) {}; 
                    return true
                }
            }
        };
        if !self.check_collision(&moved) {
            self.piece = moved;
            self.update_shadow();
            true
        } else {false}
    }

    ///updates the shadow piece
    fn update_shadow(&mut self) {
        let mut shadow = self.piece.get_shadow();
        loop {
            let moved = shadow.get_down();
            if !self.check_collision(&moved) {shadow = moved}   
            else {break}
        }
        self.shadow = shadow;
    } 

    /// does game updates
    /// returns true if gameover occurred
    pub fn update(&mut self) -> DynResult<bool> {                         
        if !self.piece_down() {                                         
            self.set_piece()?;                                          
            let cleared = self.update_rows();
            self.update_score(cleared)?;
            if self.check_game_over() {
                Ok(true)
            } else {
                if let Err(e) = self.next_piece() {
                    dynmatch!(e,
                        type TetrisError {
                            arm TetrisError::SpawnError(_) => {
                                log!(format!("{} Assuming game over", e), "debug.log");
                                Ok(true)
                            },
                            _ => Err(e)
                        },
                        _ => Err(e)
                    )
                } else {Ok(false)}
            }
        } else {Ok(false)}
    }

    ///updates score on board and in file
    fn update_score(&mut self, cleared: Vec<usize>) -> DynResult<()> {
        let modifier = match cleared.len() {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 3600
        };
        self.score += cleared.iter().map(|row|
            modifier*(BOARD_HEIGHT-row+1)
        ).collect::<Vec<_>>().iter().sum::<usize>();
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

    ///resets board
    pub fn reset(&mut self) -> DynResult<()> {
        *self = Self::new_board()?;
        Ok(())
    }

    ///checks if top row has set blocks
    fn check_game_over(&self) -> bool {
        if self.data[0].iter().any(|b| b.is_some()) {true}              //UNCHECKED INDEX
        else {false}
    }

    ///check if row is full
    fn check_full(&mut self, row: usize) -> bool {
        if self.data[row].iter().any(|b| b.is_none()) {false}
        else {true}
    }

    ///deletes row in vec and adds row at top
    fn clear_row(&mut self, row: usize) {
        self.data.remove(row);
        self.data.insert(0, vec!(None;self.data[0].len()))              //UNCHECKED INDEX
    }

    ///checks for filled rows and removes them
    fn update_rows(&mut self) -> Vec<usize> {
        let mut cleared = Vec::new();
        for row in 0..self.data.len() {
            if self.check_full(row) {
                self.clear_row(row);
                cleared.push(row);
            }
        }
        cleared
    }

    ///spawns a new piece
    fn next_piece(&mut self) -> DynResult<()> {
        if !self.check_collision(&self.next_piece) {
            self.piece = self.next_piece.clone();
            while self.next_piece.type_ == self.piece.type_ {
                self.next_piece = Piece::gen_random(self.spawn)?;
            }
            self.update_shadow();
            Ok(())
        }
        else {dynerr!(TetrisError::SpawnError(self.spawn))}
    }

    ///attempts to set piece
    fn set_piece(&mut self) -> DynResult<()>{
        for row in 0..self.piece.data.len() {
            for block in 0..self.piece.data[row].len() {
                if let Some(_) = self.piece.data[row][block] {
                    if let Some(y) = self.data.get_mut((self.piece.location.1+row as isize) as usize) {                //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                        if let Some(x) = y.get_mut((self.piece.location.0+block as isize) as usize) {                  //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                            *x = self.piece.data[row][block].clone();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    ///takes a piece and checks its collision on the board
    fn check_collision(&self, piece: &Piece) -> bool {
        for row in 0..piece.data.len() {
            for block in 0..piece.data[row].len() {
                if let Some(_) = piece.data[row][block] {
                    if let Some(y) = self.data.get((piece.location.1+row as isize) as usize) {          //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                        if let Some(x) = y.get((piece.location.0+block as isize) as usize) {            //IF ITS NEG IT'LL WRAP AND STILL BE INVALID
                            if let Some(_) = x {return true}
                        } else {return true}
                    } else {return true}
                }
            }
        }
        false
    }

    ///draws screen during game play
    pub fn draw(&self, screen: &mut engine::drawing::Screen, speed: usize){
        screen.wipe();
        screen.draw_sprite(&self.backdrop, (0,0));
        //draw set blocks
        for row in 0..self.data.len() {
            for block in 0..self.data[row].len() {
                if let Some(sprite) = &self.data[row][block] {
                    screen.draw_sprite(sprite, (((block*sprite.width)+self.padding) as isize, (row*sprite.height) as isize))
                }
            }
        }

        let mut draw_piece = |piece: &Piece, location: (isize, isize), padding: usize| {
            for row in 0..piece.data.len() {
                for block in 0..piece.data[row].len() {
                    if let Some(sprite) = &piece.data[row][block] {
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
        draw_piece(&self.shadow, self.shadow.location, self.padding);
        draw_piece(&self.piece, self.piece.location, self.padding);
        draw_piece(&self.next_piece, NEXT_PIECE_LOCATION, 0);
        if let Some(held) = &self.held_piece {
            draw_piece(held, HELD_PIECE_LOCATION, 0);
        }

        screen.draw_text((9,191), &format!("{}",self.highscore), 32.0, &[255;4], drawing::DEBUG_FONT);
        screen.draw_text((9,254), &format!("{}",self.score), 32.0, &[255;4], drawing::DEBUG_FONT);
        screen.draw_text((9,318), &format!("{}",speed), 32.0, &[255;4], drawing::DEBUG_FONT);
    }

    ///draws gameover screen
    pub fn draw_gameover(
        &self,
        screen: &mut drawing::Screen,
        speed: usize,
        second: usize
    ) {
        self.draw(screen, speed);

        let message = "GAME OVER";
        screen.draw_text((195 ,40), &message, 64.0, &GAME_OVER_COLOR, drawing::DEBUG_FONT);

        let message = format!("Score: {}",self.score);
        screen.draw_text((225,115), &message, 32.0, &GAME_OVER_COLOR, drawing::DEBUG_FONT);
    
        let message = format!("{}", second);
        screen.draw_text((290,200), &message, 128.0, &GAME_OVER_COLOR, drawing::DEBUG_FONT);
    }
}