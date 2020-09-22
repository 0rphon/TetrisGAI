use engine::sprite::Sprite;

use rand::Rng;

///the size of each block. used to calc grid
pub const BLOCK_SIZE:       usize           = 32;
///the thickness of piece border in pixels
const BORDER_SIZE:          usize           = 2;
///the color of piece borders
const BORDER_COLOR:         [u8;4]          = [0x00, 0x00, 0x00, 0xFF];
///the color of shadow
const SHADOW_COLOR:         [u8;4]          = [0x00;4];
///the color of the shadows border
const SHADOW_BORDER_COLOR:  [u8;4]          = [0xDC, 0xDC, 0xDC, 0xFF];     //[0X00;4] TO USE THE PIECE COLOR FOR SHADOW BORDER




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


///blocks in piece
pub type BlockData = Vec<Vec<Option<Sprite>>>;
///piece types
#[derive(Clone, PartialEq)]
pub enum PieceType {I, O, T, S, Z, J, L}
///the piece object
#[derive(Clone)]
pub struct Piece {                                    //ONLY PUBLIC BECAUSE BENCHMARKING REQUIRED IT TO BE
    pub type_: PieceType,
    pub location: (isize, isize),
    pub data: BlockData,
    pub can_hold: bool,
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
    fn gen_piece(shape: Vec<Vec<bool>>, color: [u8;4]) -> BlockData {
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
    pub fn gen_random(location: (isize, isize)) -> Self {
        match rand::thread_rng().gen_range(0, 7) {
            0 => {Self::new_i(location)},
            1 => {Self::new_j(location)},
            2 => {Self::new_l(location)},
            3 => {Self::new_o(location)},
            4 => {Self::new_s(location)},
            5 => {Self::new_t(location)},
            _ => {Self::new_z(location)},
        }
    }

    ///gets the shadow of a piece
    pub fn get_shadow(&self) -> Piece {
        let mut shadow = self.clone();
        for row in 0..shadow.data.len() {
            for block in 0..shadow.data[row].len() {
                if let Some(sprite) = &shadow.data[row][block] {
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
    pub fn get_rotated(&self) -> Piece {
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

    ///resets piece data to original template
    pub fn reset_rotation(&mut self) {
        self.data = {
            match self.type_ {
                PieceType::I => Self::gen_piece(fit!(I_DATA), I_COLOR),
                PieceType::J => Self::gen_piece(fit!(J_DATA), J_COLOR),
                PieceType::L => Self::gen_piece(fit!(L_DATA), L_COLOR),
                PieceType::O => Self::gen_piece(fit!(O_DATA), O_COLOR),
                PieceType::T => Self::gen_piece(fit!(T_DATA), T_COLOR),
                PieceType::S => Self::gen_piece(fit!(S_DATA), S_COLOR),
                PieceType::Z => Self::gen_piece(fit!(Z_DATA), Z_COLOR),
            }
        }
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