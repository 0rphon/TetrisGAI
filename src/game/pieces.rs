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
const I_DIM: usize = 4;
const I_DATA: [bool;I_DIM*I_DIM] = [
    false, false, false, false,
    true , true , true , true ,
    false, false, false, false,
    false, false, false, false,
];
const O_COLOR: [u8;4] = [0xFF, 0xFF, 0x00, 0xFF];
const O_DIM: usize = 2;
const O_DATA: [bool;O_DIM*O_DIM] = [
    true , true,
    true , true,
];
const T_COLOR: [u8;4] = [0x80, 0x00, 0x80, 0xFF];
const T_DIM: usize = 3;
const T_DATA: [bool; T_DIM*T_DIM] = [
    false, true , false,
    true , true , true ,
    false, false, false,
];
const S_COLOR: [u8;4] = [0x00, 0x80, 0x00, 0xFF];
const S_DIM: usize = 3;
const S_DATA: [bool;S_DIM*S_DIM] = [
    false, true , true ,
    true , true , false,
    false, false, false,
];
const Z_COLOR: [u8;4] = [0xFF, 0x00, 0x00, 0xFF];
const Z_DIM: usize = 3;
const Z_DATA: [bool;Z_DIM*Z_DIM] = [
    true , true , false,
    false, true , true ,
    false, false, false,
];
const J_COLOR: [u8;4] = [0x00, 0x00, 0xFF, 0xFF];
const J_DIM: usize = 3;
const J_DATA: [bool;J_DIM*J_DIM] = [
    true , false, false,
    true , true , true ,
    false, false, false,
];
const L_COLOR: [u8;4] = [0xFF, 0xA5, 0x00, 0xFF];
const L_DIM: usize = 3;
const L_DATA: [bool;L_DIM*L_DIM] = [
    false, false, true ,
    true , true , true ,
    false, false, false,
];

//TODO why isnt this a struct???
///list of piece info (Sprite, data, piece dimensions)
pub type PieceIndex = HashMap<PieceType, (Sprite, Vec<bool>, usize)>;
///piece types
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PieceType {I, O, T, S, Z, J, L, Shadow}

impl PieceType {
    
    //only run during board creating so i didnt bother benchmarking
    //TODO instead of iterating through entire 2d vec, convert top and bottom rows, then convert left and right portions of rows. maybe even do in 1D via chunking? pound out the first and last rows as one slice then the middle rows do chunking over its slice
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
    fn gen_piece_entry(&self) -> (Sprite, Vec<bool>, usize) {
        match *self {
            Self::I      => (Self::gen_block(I_COLOR, BORDER_COLOR), I_DATA.to_vec(), I_DIM),
            Self::J      => (Self::gen_block(J_COLOR, BORDER_COLOR), J_DATA.to_vec(), J_DIM),
            Self::L      => (Self::gen_block(L_COLOR, BORDER_COLOR), L_DATA.to_vec(), L_DIM),
            Self::O      => (Self::gen_block(O_COLOR, BORDER_COLOR), O_DATA.to_vec(), O_DIM),
            Self::T      => (Self::gen_block(T_COLOR, BORDER_COLOR), T_DATA.to_vec(), T_DIM),
            Self::S      => (Self::gen_block(S_COLOR, BORDER_COLOR), S_DATA.to_vec(), S_DIM),
            Self::Z      => (Self::gen_block(Z_COLOR, BORDER_COLOR), Z_DATA.to_vec(), Z_DIM),
            Self::Shadow => (Self::gen_block(SHADOW_COLOR, SHADOW_BORDER_COLOR), Vec::new(), 0),
        }
    }
    
    //not benched
    ///gets a random piece type
    pub fn pick_random() -> Self {
        match rand::thread_rng().gen_range(0, 7) {
            0 => Self::I,
            1 => Self::J,
            2 => Self::L,
            3 => Self::O,
            4 => Self::T,
            5 => Self::S,
            _ => Self::Z,
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


///blocks in piece
///the piece object
#[derive(Clone)]
pub struct Piece {                                    //ONLY PUBLIC BECAUSE BENCHMARKING REQUIRED IT TO BE
    pub type_: PieceType,
    pub location: (isize, isize),
    pub data: Vec<bool>,
    pub dim: usize,
    pub can_hold: bool,
}
impl Piece {

    ///generates the given piece type at the given location
    pub fn gen_piece(type_: PieceType, location: (isize, isize), index: &PieceIndex) -> Self {
        let reference = index.get(&type_).unwrap();
        Self {
            type_,
            location,
            data: reference.1.clone(),
            dim: reference.2,
            can_hold: true,
        }
    }

    ///gets a rotated version of the piece
    pub fn get_rotated(&self) -> Piece {
        let mut r = self.clone();
        for (i, block) in self.data.iter().enumerate() {
            let row = i/self.dim;
            let col = i%self.dim;
            r.data[(col*self.dim)+self.dim-row-1] = *block;
        }
        r
    }

    ///resets piece data to original template
    pub fn reset_rotation(&mut self, index: &PieceIndex) {
        self.data = index.get(&self.type_).unwrap().1.clone()
    }

    ///gets a moved version of the piece
    pub fn get_down(&self) -> (isize, isize) {
        (self.location.0, self.location.1+1)
    }

    ///gets a moved version of the piece
    pub fn get_left(&self) -> (isize, isize) {
        (self.location.0-1, self.location.1)
    }

    ///gets a moved version of the piece
    pub fn get_right(&self) -> (isize, isize) {
        (self.location.0+1, self.location.1)
    }
}






//piece data must stay on heap because pieces are variably sized...but i wonder if board data could be put to stack... 
//have master lookup table of piece sprites for drawing. generated on new board and kept at board
//as i optimize this, stripping for AI may become useless

//unnecessary clones in movement functions. need to change how thats all done
//make check collision have two values, piece and location
//that way you can separate piece from location and then just update location on final instead of cloning

//change shadow to a location instead of a full on piece then just draw shadow blocks at that location