use dynerr::*;
use engine::sprite::Sprite;

use rand::Rng;
use std::{fmt, error};
use std::fs::OpenOptions;
use std::io::prelude::*;

pub const BLOCK_SIZE: usize = 32;
pub const BORDER_SIZE: usize = 5;
pub const STANDARD_WIDTH: usize = 10;
pub const STANDARD_HEIGHT: usize = 20;

const I_DATA: [&[bool;4];4] = [
    &[false, false, false, false],
    &[true , true , true , true ],
    &[false, false, false, false],
    &[false, false, false, false],
];
const O_DATA: [&[bool;2];2] = [
    &[true , true],
    &[true , true],
];
const T_DATA: [&[bool;3];3] = [
    &[false, true , false],
    &[true , true , true ],
    &[false, false, false],
];
const S_DATA: [&[bool;3];3] = [
    &[false, true , true ],
    &[true , true , false],
    &[false, false, false],
];
const Z_DATA: [&[bool;3];3] = [
    &[true , true , false],
    &[false, true , true ],
    &[false, false, false],
];
const J_DATA: [&[bool;3];3] = [
    &[true , false, false],
    &[true , true , true ],
    &[false, false, false],
];
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











type PieceData = Vec<Vec<Option<Sprite>>>;
#[derive(Clone)]
enum PieceType {I, O, T, S, Z, J, L}
#[derive(Clone)]
struct Piece {
    type_: PieceType,
    location: (isize, isize),
    data: PieceData,
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
                    block[row_i][pixel_i] = [0xFF;4];
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
            data: Self::gen_piece(fit!(I_DATA), [0xc7, 0xf6, 0xf3, 0xFF])
        }
    }

    ///generates an O piece
    fn new_o(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::O,
            location,
            data: Self::gen_piece(fit!(O_DATA), [0xf7, 0xf0, 0xae, 0xFF]),
        }
    }

    ///generates a T piece
    fn new_t(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::T,
            location,
            data: Self::gen_piece(fit!(T_DATA), [0xdc, 0xbd, 0xf8, 0xFF]),
        }
    }

    ///generates an S piece
    fn new_s(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::S,
            location,
            data: Self::gen_piece(fit!(S_DATA), [0xc1, 0xf8, 0xb1, 0xFF]),
        }
    }

    ///generates a Z piece
    fn new_z(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::Z,
            location,
            data: Self::gen_piece(fit!(Z_DATA), [0xee, 0x72, 0x7e, 0xFF]),
        }
    }

    ///generates a J piece
    fn new_j(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::J,
            location,
            data: Self::gen_piece(fit!(J_DATA), [0x98, 0xa7, 0xee, 0xFF]),
        }
    }

    ///generates an L piece
    fn new_l(location: (isize, isize)) -> Self {
        Self {
            type_: PieceType::L,
            location,
            data: Self::gen_piece(fit!(L_DATA), [0xe4, 0xab, 0x7f, 0xFF]),
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












enum Move {
    Up,
    Down,
    Left,
    Right
}

pub struct Board {
    piece:  Piece,
    spawn: (isize, isize),
    width: usize,
    height: usize,
    data:   Vec<Vec<Option<Sprite>>>,
    pub score: usize,
    pub highscore: usize,
}

impl Board {
    ///attempts to create a new standard sized board
    pub fn new_standard() -> DynResult<Self> {
        Ok(Self {
            piece: Piece::gen_random((STANDARD_WIDTH as isize/2, 0))?,
            spawn: (STANDARD_WIDTH as isize/2, 0),
            width: STANDARD_WIDTH,
            height: STANDARD_HEIGHT,
            data: vec!(vec!(None; STANDARD_WIDTH); STANDARD_HEIGHT),
            score: 0,
            highscore: Self::get_highscore()?
        })
    }

    ///attempts to create a new custom sized board
    #[allow(unused)]
    pub fn new_custom(width: usize, height: usize) -> DynResult<Self> {
        Ok(Self {
            piece: Piece::gen_random((width as isize/2, 0))?,
            spawn: (width as isize/2, 0),
            width,
            height,
            data: vec!(vec!(None; width); height),
            score: 0,
            highscore: Self::get_highscore()?
        })
    }

    ///attempts to rotate the piece
    pub fn try_rotate(&mut self) -> bool{
        let rotated = self.piece.get_rotated();
        if !self.check_collision(&rotated) {
            self.piece = rotated;
            true
        } else {false}
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
    pub fn drop_piece(&mut self) {
        while self.piece_down() {}
    }

    /// does game updates
    /// returns true if gameover occurred
    pub fn update(&mut self) -> DynResult<bool> {                         
        if !self.piece_down() {                                         
            self.set_piece()?;                                          
            let cleared = self.update_rows();
            self.update_score(cleared)?;
            if self.check_game_over() {
                self.game_over()?;
                Ok(true)
            } else {
                if let Err(e) = self.spawn_piece() {
                    dynmatch!(e,
                        type TetrisError {
                            arm TetrisError::SpawnError(_) => {
                                log!(format!("{} Assuming game over", e), "tetris.log");
                                self.game_over()?;
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
            1 => 50,
            2 => 150,
            3 => 350,
            4 => 1000,
            _ => 2000
        };
        self.score += cleared.iter().map(|row|
            modifier*(self.height-row+1)
        ).collect::<Vec<_>>().iter().sum::<usize>();
        if self.score > self.highscore {
            self.highscore = self.score;
            self.save_highscore()?;
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

    ///saves high score to "highscore"
    fn save_highscore(&self) -> DynResult<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("highscore")?;
        file.write(format!("{}",self.highscore).as_bytes())?;
        Ok(())
    }

    ///resets board
    fn game_over(&mut self) -> DynResult<()> {
        *self = Self::new_custom(self.width, self.height)?;
        Ok(())
    }

    ///checks if top row has set blocks
    fn check_game_over(&self) -> bool {
        if !self.check_empty(0) {true}
        else {false}
    }

    ///checks if row has any blocks
    fn check_empty(&self, row: usize) -> bool {
        if self.data[row].iter().any(|b| b.is_some()) {false}
        else {true}
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

    ///checks for filled rows and removes then
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
    fn spawn_piece(&mut self) -> DynResult<()> {
        let piece = Piece::gen_random(self.spawn)?;
        if !self.check_collision(&piece) {
            println!("spawned");
            self.piece = piece;
            Ok(())
        }
        else {dynerr!(TetrisError::SpawnError(self.spawn))}
    }

    //attempts to move piece. returns bool
    fn move_piece(&mut self, direction: Move) -> bool {
        let moved = {
            match direction {
                Move::Up    => self.piece.get_up(),
                Move::Down  => self.piece.get_down(),
                Move::Left  => self.piece.get_left(),
                Move::Right => self.piece.get_right(),

            }
        };
        if !self.check_collision(&moved) {
            self.piece = moved;
            true
        } else {false}
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
        println!("{:?}",piece.location);
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

    pub fn draw(&self, screen: &mut engine::drawing::Screen){
        for row in 0..self.data.len() {
            for block in 0..self.data[row].len() {
                if let Some(sprite) = &self.data[row][block] {
                    screen.draw_sprite(sprite, ((block*sprite.width) as isize, (row*sprite.height) as isize))
                }
            }
        }
        for row in 0..self.piece.data.len() {
            for block in 0..self.piece.data[row].len() {
                if let Some(sprite) = &self.piece.data[row][block] {
                    screen.draw_sprite(
                        sprite, 
                        (
                            self.piece.location.0*sprite.width as isize + (block*sprite.width) as isize,
                            self.piece.location.1*sprite.height as isize + (row*sprite.height) as isize
                        )
                    )
                }
            }
        }
    }
}