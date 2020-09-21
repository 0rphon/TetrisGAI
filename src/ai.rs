use crate::game;

use dynerr::*;

use std::{thread, fmt};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex, mpsc, PoisonError, MutexGuard};


#[derive(Clone)]
pub struct AiParameters {
    //positives
    pub points_scored_importance: f32,
    pub piece_depth_importance: f32,
    //negatives
    pub max_height_importance: f32,
    pub avg_height_importance: f32,
    pub height_variation_importance: f32,
    pub current_holes_importance: f32,
    pub max_pillar_height: usize,
    pub current_pillars_importance: f32,
}

impl fmt::Display for AiParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{:.03} : {:.03} : {:.03} : {:.03} : {:.03} : {:.03} : {} : {:.03}",
            self.points_scored_importance,
            self.piece_depth_importance,
            self.max_height_importance,
            self.avg_height_importance,
            self.height_variation_importance,
            self.current_holes_importance,
            self.max_pillar_height,
            self.current_pillars_importance
        )
    }
}


///possible piece movements
#[derive(Debug, Clone)]
pub enum Move {
    //Down,
    Left,
    Right,
    Rotate,
    Drop,
    Hold,
    Restart,
    None,
}

///current piece rotation relative to its start
#[derive(Clone, Debug)]
enum Rotation {
    North,
    South,
    East,
    West,
}

#[derive(Debug)]
struct MoveData {
    location: (isize, isize),
    is_held: bool,
    rotation: Rotation,
    board: game::StrippedData,
    value: f32,
    debug_scores: Vec<f32>,
}

impl MoveData {

    fn generate_data(mut board: game::StrippedData, piece: game::StrippedPiece, is_held: bool, rotation: Rotation, parameters: &AiParameters) -> Self {
        for (i, block) in piece.data.data.iter().enumerate() {
            if *block {
                let row = i/piece.data.width;
                let column = i%piece.data.width;
                let board_index = (((piece.location.1+row as isize)*board.width as isize) + (piece.location.0+column as isize)) as usize;           //USIZE WRAPPING
                if let Some(cell) = board.data.get_mut(board_index) {*cell = true}
            }
        }

        let mut move_data = {
            Self {
                location: piece.location,
                is_held,
                rotation,
                board,
                value: 0.0,
                debug_scores: vec!(),
            }
        };

        move_data.calc_board(parameters);
        move_data
    }

    //need to have const floats as modifiers for importance
    /// calculates the move score. the higher the score the better
    /// also calcs the next board
    fn calc_board(&mut self, parameters: &AiParameters) {
        //updates board and gets points scored
        let points_scored    = self.do_clear()*parameters.points_scored_importance;
        //gets how far down the piece was placed
        let piece_depth      = self.location.1 as f32*parameters.piece_depth_importance;                                           //y location should always be positive
        //gets heights of every column
        let column_heights   = self.get_heights();
        //tallest column
        let max_height       = *column_heights.last().unwrap() as f32*parameters.max_height_importance;                                        //DIRECT UNWRAP
        //average column height
        let avg_height       = (column_heights.iter().sum::<usize>() as f32/column_heights.len() as f32)*parameters.avg_height_importance;
        //tallest column - smallest column
        let height_variation = ((column_heights.last().unwrap_or(&self.board.height)-column_heights.first().unwrap_or(&0)) as f32)*parameters.height_variation_importance;
        //how many gaps exist in columns
        let current_holes    = self.calc_holes()*parameters.current_holes_importance;
        //how many spots where empty spaces surrounded by filled spaces on either side exist (over the set max allowed pillar height)
        let current_pillars  = self.calc_pillars(parameters.max_pillar_height)*parameters.current_pillars_importance;

        self.debug_scores = vec!(points_scored, piece_depth, max_height, avg_height, height_variation, current_holes, current_pillars);
        self.value = points_scored+piece_depth-max_height-avg_height-height_variation-current_holes-current_pillars;
    }

    /// returns a list of all column heights.
    fn get_heights(&self) -> Vec<usize> {
        let mut heights = Vec::new();
        for x in 0..self.board.width {
            let mut idx = x;
            for y in 0..self.board.height {
                if self.board.data[idx] {
                    heights.push((self.board.height-y) as usize);
                    break
                } else if y+1 == self.board.height {
                    heights.push(0);
                    break
                }
                idx += self.board.width;
            }
        }
        heights.sort();
        heights
    }

    ///how many empty spaces have blocks over them
    fn calc_holes(&self) -> f32 {
        let mut holes = 0;
        for x in 0..self.board.width {
            let mut idx = x;
            let mut under = false;
            for _ in 0..self.board.height {
                if self.board.data[idx] {under = true}
                else if !self.board.data[idx]
                && under {holes+=1}
                idx += self.board.width;
            }
        }
        holes as f32
    }

    ///each additional block for pillars over 2 blocks
    fn calc_pillars(&self, max_pillar_height: usize) -> f32 {
        let mut pillars = 0;
        for x in 0..self.board.width {
            let mut idx = x;
            let mut pillar_height = 0;
            for _ in 0..self.board.height {
                if !self.board.data[idx]
                && (*self.board.data
                        .get((idx)
                            .checked_sub(1)
                            .unwrap_or(9999)
                        )                                                                                               //BAD SOLUTION
                        .unwrap_or(&true)
                    || x == 0                                                                                           //CHECK IF EDGE OF SCREEN
                ) && (*self.board.data
                        .get(idx)
                        .unwrap_or(&true)
                    || x == self.board.width-1                                                                          //CHECK IF EDGE OF SCREEN
                ) {
                    pillar_height+=1
                }
                idx += self.board.width;
            }
            if pillar_height > max_pillar_height {pillars+=pillar_height-max_pillar_height}
        }
        pillars as f32
    }

    //if need be, i could make this return the exact rows cleared so AI could go after higher rows?
    ///clears rows, adds new empty rows, and returns points scored
    fn do_clear(&mut self) -> f32 {
        let mut cleared = Vec::new();
        for y in 0..self.board.height {
            let start_range = y*self.board.width;
            let end_range = start_range+self.board.width;
            if self.board.data[start_range..end_range].iter().all(|b| *b) {
                self.board.data.drain(start_range..end_range);
                self.board.data.splice(0..0, vec!(false;self.board.width));
                cleared.push(self.board.height-y)
            }
        }
        let modifier = match cleared.len() {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 3600
        };
        cleared.iter().map(|y|modifier*(y+1)).sum::<usize>() as f32
    }

    fn gen_input(&self, board: &game::StrippedBoard) -> Vec<Move>{
        let mut moves = Vec::new();
        let piece = {
            if self.is_held {
                moves.push(Move::Hold);
                if let Some(held) = &board.held_piece {
                    held
                } else {&board.next_piece}
            } else {&board.piece}
        };
        ////LOGGING##################################################################################################
        //log!(format!("target {:?}, {:?} got score: {}", self.location, self.rotation, self.value), "ai.log");   //#
        //let mut scores = String::new();                                                                         //#
        //for score in &self.debug_scores {scores.push_str(&format!("{}, ", score))}                              //#
        //log!(scores, "ai.log");                                                                                 //#
        //for row in self.board.data.chunks(self.board.width) {                                                   //#
        //    let mut r = String::new();                                                                          //#
        //    for column in row {                                                                                 //#
        //        if *column {                                                                                    //#
        //            r.push_str("[X]")                                                                           //#
        //        } else {r.push_str("[ ]")}                                                                      //#
        //    }                                                                                                   //#
        //    log!(r, "ai.log");                                                                                  //#
        //}                                                                                                       //#
        ////#########################################################################################################
        //LOGGING######################################################################################
        //log!(format!("current: {:?}", piece.location), "ai.log");                                   //#
        //for row in piece.data.data.chunks(piece.data.width) {                                       //#
        //    let mut r = String::new();                                                              //#
        //    for column in row {                                                                     //#
        //        if *column {                                                                        //#
        //            r.push_str("[X]")                                                               //#
        //        } else {r.push_str("[ ]")}                                                          //#
        //    }                                                                                       //#
        //    log!(r, "ai.log");                                                                      //#
        //}                                                                                           //#
        ////#############################################################################################
        let roto_times = {
            match self.rotation {
                Rotation::North => 0,
                Rotation::West  => 1,
                Rotation::South => 2,
                Rotation::East  => 3,
            }
        };
        for _ in 0..roto_times {moves.push(Move::Rotate)}
        let distance = self.location.0 - piece.location.0;
        if distance > 0 {
            for _ in 0..distance {moves.push(Move::Right)}
        } else if distance < 0 {
            for _ in distance..0 {moves.push(Move::Left)}
        }
        moves.push(Move::Drop);
        ////LOGGING##########################################################
        //log!(format!("Moves {:?}\n", moves), "ai.log");                 //#
        ////#################################################################
        moves
    }
}

///rotates piece data
fn rotate_piece(piece: &mut game::StrippedData) {
    let original = piece.data.clone();
    for i in 0..piece.data.len() {
        let column = i%piece.width;
        let row = i/piece.width;
        piece.data[(row*piece.width)+column] = original[(column*piece.width)+piece.width-row-1];
    }
}

///checks piece for collision on board
fn check_collision(board: &game::StrippedData, piece: &game::StrippedPiece) -> bool {
    for i in 0..piece.data.data.len() {
        if piece.data.data[i] {
            let row = i/piece.data.width;
            let column = i%piece.data.width;
            if (piece.location.0+column as isize) < 0
            || (piece.location.0+column as isize) > board.width as isize-1
            || (piece.location.1+row as isize) < 0
            || (piece.location.1+row as isize) > board.height as isize-1
                {return true}
            if let Some(cell) = board.data.get((((piece.location.1+row as isize)*board.width as isize)+(piece.location.0+column as isize)) as usize) {                      //RELIES ON USIZE WRAPPING
                if *cell {return true}
            } else {return true}
        }
    }
    false
}

///get all possible moves for a piece
fn get_moves_for_piece(board: &game::StrippedData, mut piece: game::StrippedPiece, is_held: bool, parameters: &AiParameters) -> Vec<MoveData> {
    let mut possible_moves =  Vec::new();
    let original_location = piece.location;
    for rotation in [Rotation::North, Rotation::East, Rotation::South, Rotation::West].iter() {
        //move to left edge
        while !check_collision(board, &piece) {
            piece.location.0-=1;
        }
        piece.location.0+=1;
        //while piece in valid location
        while !check_collision(board, &piece) {
            //drop
            while !check_collision(board, &piece) {
                piece.location.1+=1;
            }
            piece.location.1-=1;
            //add move
            possible_moves.push(MoveData::generate_data(board.clone(), piece.clone(), is_held, rotation.clone(), parameters));
            //reset piece and move over one
            piece.location.1 = original_location.1;
            piece.location.0 += 1;
        }
        piece.location = original_location;
        rotate_piece(&mut piece.data);
    }
    possible_moves
}

///get all possible moves for current board
fn get_possible_moves(board: &game::StrippedBoard, parameters: &AiParameters) -> Vec<MoveData> {
    let mut possible_moves = Vec::new();
    possible_moves.extend(get_moves_for_piece(&board.data, board.piece.clone(), false, parameters));
    if board.piece.can_hold {
        if let Some(held) = &board.held_piece {
            possible_moves.extend(get_moves_for_piece(&board.data, held.clone(), true, parameters));
        } else {
            possible_moves.extend(get_moves_for_piece(&board.data, board.next_piece.clone(), true, parameters));
        }
    }
    possible_moves
}


///takes board. gets all possible moves. finds best move. generates input
fn get_input_move(board: game::StrippedBoard, parameters: &AiParameters) -> (Vec<Move>, Option<Vec<bool>>) {
    let mut possible_moves = get_possible_moves(&board, parameters);
    if !possible_moves.is_empty() {
        possible_moves.sort_by(|a,b| b.value.partial_cmp(&a.value).unwrap_or(Ordering::Equal));     //IF NAN DEFAULTS TO EQUAL
        let chosen_move = possible_moves.remove(0);
        (chosen_move.gen_input(&board), Some(chosen_move.board.data))
    } else {(vec!(Move::Restart), None)}
}






///generates a log message of board mismatch
fn log_board(last: &Vec<bool>, predicted: &Vec<bool>, board: &game::StrippedData) {
    let mut message = String::from("Board mismatch!\nLast:\n");
    for row in last.chunks(board.width) {
        for column in row {
            if *column {
                message.push_str("[X]")
            } else {message.push_str("[ ]")}
        }
        message.push('\n');
    }

    message.push_str("Expected:\n");
    for row in predicted.chunks(board.width) {
        for column in row {
            if *column {
                message.push_str("[X]")
            } else {message.push_str("[ ]")}
        }
        message.push('\n');
    }
    message.push_str("Actual:\n");
    for row in board.data.chunks(board.width) {
        for column in row {
            if *column {
                message.push_str("[X]")
            } else {message.push_str("[ ]")}
        }
        message.push('\n');
    }
    message.push('\n');
    log!(message, "ai.log");
}

pub struct Packet {
    board: Option<game::StrippedBoard>,
    exit: bool,
}

///the communicator for main thread
pub struct MainRadio {
    tx: mpsc::Sender<Packet>,
    input: Arc<Mutex<Vec<Move>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MainRadio {
    ///sends a packet to ai
    fn send(&self, message: Packet) -> DynResult<()> {
        self.tx.send(message)?;
        Ok(())
    }

    ///sends the board to ai
    pub fn send_board(&self, board: game::StrippedBoard) -> DynResult<()> {
        self.send(Packet{board: Some(board), exit: false})
    }

    ///gets next move from ai
    pub fn get_input(&self) -> Result<Option<Move>, PoisonError<MutexGuard<Vec<Move>>>>{
        let mut ai_input = self.input.lock()?;
        if ai_input.get(0).is_some() {
            Ok(Some(ai_input.remove(0)))
        } else {Ok(None)}
    }

    ///tells ai thread to exit and waits for join
    pub fn join(&mut self) -> DynResult<()> {
        self.tx.send(Packet{board: None, exit: true})?;
        self.handle.take().map(|e| e.join());
        Ok(())
    }
}

///the communicator for ai thread
struct AiRadio {
    input: Arc<Mutex<Vec<Move>>>,
    rx: mpsc::Receiver<Packet>,
}

impl AiRadio {
    ///sets the ai input
    fn set_input(&self, input: Vec<Move>) -> Result<(), PoisonError<MutexGuard<Vec<Move>>>> {
        *(self.input.lock()?) = input;
        Ok(())
    }

    /// notify the main thread that you got a board but you not generating doing any new moves
    /// only writes to buffer is buffer is empty, else does nothing.
    /// used to tell trainer function that it got the board but chose not to move
    fn dont_move(&self) -> Result<(), PoisonError<MutexGuard<Vec<Move>>>>{
        let mut ai_input = self.input.lock()?;
        if ai_input.is_empty() {*ai_input = vec!(Move::None)}
        Ok(())
    }
}

///for every packet received calculates moves
fn ai_loop(radio: AiRadio, parameters: AiParameters, log_flag: bool) {
    let mut last_board = Vec::new();
    let mut predicted_board: Option<Vec<bool>> = None;
    for packet in &radio.rx {
        if let Some(new_board) = packet.board {
            if new_board.data.data != last_board {
                if log_flag {
                    if let Some(predicted) = &predicted_board {
                        if *predicted != new_board.data.data {
                            log_board(&last_board, predicted, &new_board.data);
                        }
                    }
                }
                last_board = new_board.data.data.clone();
                if !new_board.gameover {
                    let result = get_input_move(new_board, &parameters);
                    check!(radio.set_input(result.0));
                    predicted_board = result.1;
                } else {check!(radio.set_input(vec!(Move::Restart)))}
            } else {check!(radio.dont_move())}
        } else if packet.exit {break}
    }
}

///starts the AI thread
pub fn start(parameters: AiParameters, log_flag: bool) -> MainRadio {
    if log_flag {clean!("ai.log")}
    let input = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel();
    let ai_radio = AiRadio {input: Arc::clone(&input), rx};
    let handle = thread::spawn(move || {ai_loop(ai_radio, parameters, log_flag)});
    MainRadio {tx, input, handle: Some(handle)}
}













// #[cfg(test)]
// mod tests {
//     extern crate test;
//     use super::*;

//     //132,666 ns/iter (+/- 40,042)
//     #[bench]
//     fn ai_speed(b: &mut test::Bencher) {
//         let board = check!(game::Board::new_board());
//         let parameters = AiParameters {
//             points_scored_importance:       0.50,
//             piece_depth_importance:         0.25,
//             max_height_importance:          0.75,
//             avg_height_importance:          0.0,
//             height_variation_importance:    0.5,
//             current_holes_importance:       3.5,
//             max_pillar_height:              2,
//             current_pillars_importance:     0.75,
//         };       

//         b.iter(|| 
//             get_input_move(board.get_board(), &parameters)
//         );
//     }
// }