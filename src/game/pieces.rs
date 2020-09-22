#![doc(hidden)]

use engine::sprite::Sprite;

use rand::Rng;
use std::collections::HashMap;

///the size of each block. used to calc grid
pub const BLOCK_SIZE:       usize           = 32;
///the thickness of piece border in pixels
const BORDER_SIZE:          usize           = 2;
///the color of piece borders
const BORDER_COLOR:         [u8;4]          = [0x00, 0x00, 0x00, 0xFF];
///the color of shadow
const SHADOW_COLOR:         [u8;4]          = [0x00;4];
///the color of the shadows border
const SHADOW_BORDER_COLOR:  [u8;4]          = [0xDC, 0xDC, 0xDC, 0xFF];




const I_COLOR: [u8;4] = [0x00, 0xFF, 0xFF, 0xFF];
const I_DATA: [[bool;4];4] = [
    [false, false, false, false],
    [true , true , true , true ],
    [false, false, false, false],
    [false, false, false, false],
];
const O_COLOR: [u8;4] = [0xFF, 0xFF, 0x00, 0xFF];
const O_DATA: [[bool;2];2] = [
    [true , true],
    [true , true],
];
const T_COLOR: [u8;4] = [0x80, 0x00, 0x80, 0xFF];
const T_DATA: [[bool;3];3] = [
    [false, true , false],
    [true , true , true ],
    [false, false, false],
];
const S_COLOR: [u8;4] = [0x00, 0x80, 0x00, 0xFF];
const S_DATA: [[bool;3];3] = [
    [false, true , true ],
    [true , true , false],
    [false, false, false],
];
const Z_COLOR: [u8;4] = [0xFF, 0x00, 0x00, 0xFF];
const Z_DATA: [[bool;3];3] = [
    [true , true , false],
    [false, true , true ],
    [false, false, false],
];
const J_COLOR: [u8;4] = [0x00, 0x00, 0xFF, 0xFF];
const J_DATA: [[bool;3];3] = [
    [true , false, false],
    [true , true , true ],
    [false, false, false],
];
const L_COLOR: [u8;4] = [0xFF, 0xA5, 0x00, 0xFF];
const L_DATA: [[bool;3];3] = [
    [false, false, true ],
    [true , true , true ],
    [false, false, false],
];

///converts [&[T; Size]; Size] to Vec<Vec<T>>
macro_rules! fit {
    ($x:expr) => {
        $x.iter().map(|y| y.to_vec()).collect::<Vec<_>>()
    };
}

///blocks in piece
pub type GridData = Vec<Vec<bool>>;
///list of piece info
pub type PieceIndex = HashMap<PieceType, (Sprite, Vec<Vec<bool>>)>;
///piece types
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {I, O, T, S, Z, J, L, Shadow}

impl PieceType {

    //only run during board creating so i didnt bother benchmarking
    ///generates a colored block with a border
    fn gen_block(color: [u8;4], border: [u8;4]) -> Sprite {
        let mut block = vec!(vec!(color; BLOCK_SIZE); BLOCK_SIZE);
        for row_i in 0..BLOCK_SIZE {
            for pixel_i in 0..BLOCK_SIZE {
                if (0..BORDER_SIZE).contains(&row_i)
                || (0..BORDER_SIZE).contains(&pixel_i)
                || (BLOCK_SIZE-BORDER_SIZE..BLOCK_SIZE).contains(&row_i)
                || (BLOCK_SIZE-BORDER_SIZE..BLOCK_SIZE).contains(&pixel_i) {
                    block[row_i][pixel_i] = border;
                }
            }
        }
        Sprite::add(BLOCK_SIZE, BLOCK_SIZE, block)
    }

    
    //only run during board creating so i didnt bother benchmarking
    ///generates a pieces associated info
    fn gen_piece_entry(&self) -> (Sprite, Vec<Vec<bool>>) {
        match *self {
            Self::I => (Self::gen_block(I_COLOR, BORDER_COLOR), fit!(I_DATA)),
            Self::J => (Self::gen_block(J_COLOR, BORDER_COLOR), fit!(J_DATA)),
            Self::L => (Self::gen_block(L_COLOR, BORDER_COLOR), fit!(L_DATA)),
            Self::O => (Self::gen_block(O_COLOR, BORDER_COLOR), fit!(O_DATA)),
            Self::T => (Self::gen_block(T_COLOR, BORDER_COLOR), fit!(T_DATA)),
            Self::S => (Self::gen_block(S_COLOR, BORDER_COLOR), fit!(S_DATA)),
            Self::Z => (Self::gen_block(Z_COLOR, BORDER_COLOR), fit!(Z_DATA)),
            Self::Shadow => (Self::gen_block(SHADOW_COLOR, SHADOW_BORDER_COLOR), Vec::new()),
        }
    }

    
    //only run during board creating so i didnt bother benchmarking
    ///generates hashmap index of pieces and their associated data
    pub fn gen_piece_index() -> PieceIndex {
        let mut index = HashMap::new();
        for piece in [PieceType::I, PieceType::J, PieceType::L, PieceType::O,PieceType::T,PieceType::S, PieceType::Z, PieceType::Shadow].iter() {
            assert!(index.insert(*piece, piece.gen_piece_entry()).is_none());
        }
        index
    }
}


///the piece object
#[derive(Clone)]
pub struct Piece {                                    //ONLY PUBLIC BECAUSE BENCHMARKING REQUIRED IT TO BE
    pub type_: PieceType,
    pub location: (isize, isize),
    pub data: GridData,
    pub can_hold: bool,
}
impl Piece {

    ///attempts to generate a random piece
    pub fn gen_random(location: (isize, isize), index: &PieceIndex) -> Self {
        let type_ = match rand::thread_rng().gen_range(0, 7) {
            0 => {PieceType::I},
            1 => {PieceType::J},
            2 => {PieceType::L},
            3 => {PieceType::O},
            4 => {PieceType::T},
            5 => {PieceType::S},
            _ => {PieceType::Z},
        };
        Self {
            type_,
            location,
            data: index.get(&type_).unwrap().1.clone(),
            can_hold: true,
        }
    }

    ///gets a rotated version of the piece
    pub fn get_rotated(&self) -> Piece {
        let height = self.data.len();
        let width = self.data[0].len();                                         //UNCHECKED INDEX
        let mut r = self.clone();
        for row in 0..height {
            for block in 0..width {
                if self.data[row][block] {
                    r.data[block][width-row-1] = true;
                } else {
                    r.data[block][width-row-1] = false;
                }
            }
        }
        r
    }

    ///resets piece data to original template
    pub fn reset_rotation(&mut self, index: &PieceIndex) {
        self.data = index.get(&self.type_).unwrap().1.clone()
    }

    ///gets a moved version of the piece
    pub fn get_down(&self) -> Self {
        let mut moved = self.clone();
        if let Some(y) = moved.location.1.checked_add(1) {
            moved.location.1 = y;
        }
        moved
    }

    ///gets a moved version of the piece
    pub fn get_left(&self) -> Self {
        let mut moved = self.clone();
        if let Some(x) = moved.location.0.checked_sub(1) {
            moved.location.0 = x;
        }
        moved
    }

    ///gets a moved version of the piece
    pub fn get_right(&self) -> Self {
        let mut moved = self.clone();
        if let Some(x) = moved.location.0.checked_add(1) {
            moved.location.0 = x;
        }
        moved
    }
}






//piece data must stay on heap because pieces are variably sized...but i wonder if board data could be put to stack... 
//have master lookup table of piece sprites for drawing. generated on new board and kept at board
//as i optimize this, stripping for AI may become useless

//unnecessary clones in movement functions. need to change how thats all done
//make check collision have two values, piece and location
//that way you can separate piece from location and then just update location on final instead of cloning

//change shadow to a location instead of a full on piece then just draw shadow blocks at that location