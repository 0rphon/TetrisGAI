use dynerr::*;
use engine::sprite::Sprite;

use rand::Rng;
use std::{fmt, error};

pub const BLOCK_SIZE: usize = 32;
pub const BORDER_SIZE: usize = 5;
pub const STANDARD_WIDTH: usize = 10;
pub const STANDARD_HEIGHT: usize = 20;

const I_DATA: Vec<Vec<bool>> = vec!(
    vec!(false, false, false, false),
    vec!(true , true , true , true),
    vec!(false, false, false, false),
    vec!(false, false, false, false),
);
const O_DATA: Vec<Vec<bool>> = vec!(
    vec!(true , true),
    vec!(true , true),
);
const T_DATA: Vec<Vec<bool>> = vec!(
    vec!(false, true , false),
    vec!(true , true , true ),
    vec!(false, false, false),
);
const S_DATA: Vec<Vec<bool>> = vec!(
    vec!(false, true , true ),
    vec!(true , true , false),
    vec!(false, false, false),
);
const Z_DATA: Vec<Vec<bool>> = vec!(
    vec!(true , true , false),
    vec!(false, true , true ),
    vec!(false, false, false),
);
const J_DATA: Vec<Vec<bool>> = vec!(
    vec!(true , false, false),
    vec!(true , true , true ),
    vec!(false, false, false),
);
const L_DATA: Vec<Vec<bool>> = vec!(
    vec!(false, false, true ),
    vec!(true , true , true ),
    vec!(false, false, false),
);

///a custom error type
#[derive(Debug)]
enum TetrisError {
    SpawnError((usize, usize)),
    ShapeError(usize, usize),
    GenerationError(u32),
}
//impl display formatting for error
impl fmt::Display for TetrisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TetrisError::SpawnError((x,y))      => write!(f, "TetrisError::SpawnError: failed to spawn piece at x:{} y:{}", x, y),
            TetrisError::ShapeError(x,y)        => write!(f, "TetrisError::ShapeError: shape dimensions x:{} y:{} different sizes!", x, y),
            TetrisError::GenerationError(i)     => write!(f, "PieceError::GenerationError: random number generator returned invalid value: {}", i),
        }
    }
}
//impl error conversion for error
impl error::Error for TetrisError {}












type PieceData = Vec<Vec<Option<Sprite>>>;
#[derive(Clone)]
enum PieceType {I, O, T, S, Z, J, L}
enum Rotation {Up, Down, Left, Right}
#[derive(Clone)]
struct Piece {
    type_: PieceType,
    location: (usize, usize),
    data: PieceData,
}
impl Piece {
    ///generates a colored block with a border
    fn gen_block(color: [u8;4]) -> Sprite {
        let mut block = vec!(vec!(color; BLOCK_SIZE); BLOCK_SIZE);
        for row_i in 0..BLOCK_SIZE {
            for pixel_i in 0..BLOCK_SIZE {
                if (0..BORDER_SIZE).contains(&row_i) 
                || (0..BORDER_SIZE).contains(&pixel_i) {
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
    fn new_i(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::I,
            location,
            data: Self::gen_piece(I_DATA, [0x00;4])
        }
    }

    ///generates an O piece
    fn new_o(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::O,
            location,
            data: Self::gen_piece(O_DATA, [0x00;4]),
        }
    }

    ///generates a T piece
    fn new_t(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::T,
            location,
            data: Self::gen_piece(T_DATA, [0x00;4]),
        }
    }

    ///generates an S piece
    fn new_s(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::S,
            location,
            data: Self::gen_piece(S_DATA, [0x00;4]),
        }
    }

    ///generates a Z piece
    fn new_z(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::Z,
            location,
            data: Self::gen_piece(Z_DATA, [0x00;4]),
        }
    }

    ///generates a J piece
    fn new_j(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::J,
            location,
            data: Self::gen_piece(J_DATA, [0x00;4]),
        }
    }

    ///generates an L piece
    fn new_l(location: (usize, usize)) -> Self {
        Self {
            type_: PieceType::L,
            location,
            data: Self::gen_piece(L_DATA, [0x00;4]),
        }
    }

    ///attempts to generate a random piece
    fn gen_random(location: (usize, usize)) -> DynResult<Self> {
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

    fn get_rotated(&self) -> Piece {
        let height = self.data.len();
        let width = self.data[0].len();
        let mut r = self.clone();
        for row in 0..height {
            for block in 0..width {
                if let Some(sprite) = &self.data[row][block] {
                    r.data[block][width-row-1] = Some(sprite.clone())
                }
            }
        }
        r
    }

    fn get_up(&self) -> Self {
        let mut moved = self.clone();
        if let Some(y) = moved.location.1.checked_add(1) {
            moved.location.1 = y;
        }
        moved
    }

    fn get_down(&self) -> Self {
        let mut moved = self.clone();
        if let Some(y) = moved.location.1.checked_sub(1) {
            moved.location.1 = y;
        }
        moved
    }

    fn get_left(&self) -> Self {
        let mut moved = self.clone();
        if let Some(x) = moved.location.0.checked_sub(1) {
            moved.location.0 = x;
        }
        moved
    }

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
    spawn: (usize, usize),
    width:  usize,
    height: usize,
    data:   Vec<Vec<Option<Sprite>>>
}

impl Board {
    ///attempts to create a new standard sized board
    pub fn new_standard() -> DynResult<Self> {
        Ok(Self {
            piece: Piece::gen_random((STANDARD_WIDTH, STANDARD_HEIGHT))?,
            spawn: (STANDARD_WIDTH, STANDARD_HEIGHT),
            width: STANDARD_WIDTH,
            height: STANDARD_HEIGHT,
            data: vec!(vec!(None; STANDARD_WIDTH); STANDARD_HEIGHT)
        })
    }

    ///attempts to create a new custom sized board
    pub fn new_custom(width: usize, height: usize) -> DynResult<Self> {
        Ok(Self {
            piece: Piece::gen_random((width/2, height))?,
            spawn: (width/2, height),
            width,
            height,
            data: vec!(vec!(None; width); height)
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
    fn piece_up(&mut self) -> bool {
        self.move_piece(Move::Up)
    }

    /// attempts to move piece down
    /// sets piece if cant move down
    pub fn piece_down(&mut self) -> bool{
        if !self.move_piece(Move::Down) {
            self.set_piece();
            false
        }
        else {true}
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

    ///does game updates
    pub fn update(&mut self) {                          //NEEDS ROW CLEAR, SCORE, GAME OVER, ETC
        self.piece_down();
    }

    ///spawns a new piece
    fn spawn_piece(&mut self) -> DynResult<()> {
        let piece = Piece::gen_random(self.spawn)?;
        if self.check_collision(&piece) {
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
        if self.check_collision(&moved) {
            self.piece = moved;
            true
        } else {false}
    }

    ///attempts to set piece
    fn set_piece(&mut self) -> DynResult<()>{
        for row in 0..self.piece.data.len() {
            for block in 0..self.piece.data[row].len() {
                if let Some(y) = self.data.get_mut(self.piece.location.1+row) {
                    if let Some(x) = y.get_mut(self.piece.location.0+block) {
                        *x = self.piece.data[row][block].clone();
                    }
                }
            }
        }
        self.spawn_piece()?;
        Ok(())
    }

    ///takes a piece and checks its collision on the board
    fn check_collision(&self, piece: &Piece) -> bool {
        for row in 0..piece.data.len() {
            for block in 0..piece.data[row].len() {
                if let Some(y) = self.data.get(piece.location.1+row) {
                    if let Some(x) = y.get(piece.location.0+block) {
                        if let Some(_) = x {return true}
                    }
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
    }
}