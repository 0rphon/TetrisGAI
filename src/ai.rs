use crate::tetris;

use dynerr::*;

use std::{thread, fmt};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex, mpsc, PoisonError, MutexGuard};


#[derive(Clone)]
pub struct AiParameters {
    //positives
    ///min number of rows to reward clearing
    pub min_cleared_rows: usize,
    pub cleared_rows_importance: f32,
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
            "{} : {:.03} : {:.03} : {:.03} : {:.03} : {:.03} : {:.03} : {} : {:.03}", 
            self.min_cleared_rows, 
            self.cleared_rows_importance,
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
    board: tetris::StrippedData,
    score: f32,
    debug_scores: Vec<f32>,
}

impl MoveData {

    fn generate_data(mut board: tetris::StrippedData, piece: tetris::StrippedPiece, is_held: bool, rotation: Rotation, parameters: &AiParameters) -> Self {
        for row in 0..piece.data.len() {
            for column in 0..piece.data[row].len() {
                if piece.data[row][column] {
                    if let Some(y) = board.get_mut((piece.location.1+row as isize) as usize) {                      //RELIES ON USIZE WRAPPING
                        if let Some(x) = y.get_mut((piece.location.0+column as isize) as usize) {                   //RELIES ON USIZE WRAPPING
                            *x = true
                        }
                    }
                }
            }
        }

        let mut move_data = {
            Self {
                location: piece.location,
                is_held,
                rotation,
                board,
                score: 0.0,
                debug_scores: vec!(),
            }
        };

        move_data.calc_score(parameters);
        move_data
    }

    //need to have const floats as modifiers for importance
    ///calculates the move score. the higher the score the better
    fn calc_score(&mut self, parameters: &AiParameters) {
        let cleared_rows     = match self.calc_cleared() {
            i if i == 0.0 => 0.0, 
            i if i < parameters.min_cleared_rows as f32 => (i*parameters.cleared_rows_importance)*-1.0, 
            i => (i-parameters.min_cleared_rows as f32+1.0)*parameters.cleared_rows_importance,
        };
        let max_height       = self.calc_max_height()*parameters.max_height_importance;
        let avg_height       = self.calc_avg_height()*parameters.avg_height_importance;
        let height_variation = self.calc_height_variation()*parameters.height_variation_importance;
        let current_holes    = self.calc_holes()*parameters.current_holes_importance;
        let current_pillars  = self.calc_pillars(parameters.max_pillar_height)*parameters.current_pillars_importance;
        let piece_depth      = self.location.1 as f32*parameters.piece_depth_importance;                                           //y location should always be positive
        self.debug_scores = vec!(cleared_rows, piece_depth, max_height, avg_height, height_variation, current_holes, current_pillars);
        self.score = cleared_rows+piece_depth-max_height-avg_height-height_variation-current_holes-current_pillars;
    }

    ///returns how many empty rows from top
    fn calc_max_height(&self) -> f32 {
        for y in 0..self.board.len() {
            for x in 0..self.board[y].len() {
                if self.board[y][x] {
                    return (self.board.len()-y) as f32
                }
            }
        }
        0.0
    }

    ///returns average empty rows from top
    fn calc_avg_height(&self) -> f32 {
        let mut heights = Vec::new();
        for x in 0..self.board[0].len(){                                //UNCHECKED INDEX
            for y in 0..self.board.len() {
                if self.board[y][x] {
                    heights.push(self.board.len()-y);
                    break
                }
            }
        }
        heights.iter().sum::<usize>() as f32/heights.len() as f32
    }

    ///difference between lowest and tallest columns
    fn calc_height_variation(&self) -> f32 {
        let mut heights = Vec::new();
        for x in 0..self.board[0].len(){                                //UNCHECKED INDEX
            for y in 0..self.board.len() {
                if self.board[y][x] {
                    heights.push(self.board.len()-y);
                    break
                }
            }
        }
        heights.sort();
        (heights.last().unwrap_or(&self.board.len())-heights.first().unwrap_or(&0)) as f32
    }

    ///how many empty spaces have blocks over them
    fn calc_holes(&self) -> f32 {
        let mut holes = 0;
        for x in 0..self.board[0].len() {                               //UNCHECKED INDEX
            let mut under = false;
            for y in 0..self.board.len() {
                if self.board[y][x] {under = true}
                else if !self.board[y][x] 
                && under {holes+=1}
            }
        }
        holes as f32
    }

    ///each additional block for pillars over 2 blocks
    fn calc_pillars(&self, max_pillar_height: usize) -> f32 {
        let mut pillars = 0;
        for x in 0..self.board[0].len(){                                                    //UNCHECKED INDEX
            let mut pillar_height = 0;
            for y in 0..self.board.len() {
                if !self.board[y][x]
                && *self.board[y].get(x.checked_sub(1).unwrap_or(99)).unwrap_or(&true)      //BAD SOLUTION
                && *self.board[y].get(x+1).unwrap_or(&true) {
                    pillar_height+=1;
                }
            }
            if pillar_height > max_pillar_height {pillars+=pillar_height-max_pillar_height}
        }
        pillars as f32
    }

    //returns how many rows cleared
    fn calc_cleared(&self) -> f32 {
        let mut cleared = 0;
        for y in &self.board {
            if y.iter().all(|b| *b) {
                cleared+=1;
            }
        }
        cleared as f32
    }

    fn gen_input(&self, board: &tetris::StrippedBoard) -> Vec<Move>{
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
        //log!(format!("target {:?}, {:?} got score: {}", self.location, self.rotation, self.score), "ai.log");   //#
        //let mut scores = String::new();                                                                         //#
        //for score in &self.debug_scores {scores.push_str(&format!("{}, ", score))}                              //#
        //log!(scores, "ai.log");                                                                                 //#
        //for row in &self.board {                                                                                //#
        //    let mut r = String::new();                                                                          //#
        //    for column in row {                                                                                 //#
        //        if *column {                                                                                    //#
        //            r.push_str("[X]")                                                                           //#
        //        } else {r.push_str("[ ]")}                                                                      //#
        //    }                                                                                                   //#
        //    log!(r, "ai.log");                                                                                  //#
        //}                                                                                                       //#
        ////#########################################################################################################
        ////LOGGING######################################################################################
        //log!(format!("current: {:?}", piece.location), "ai.log");                                   //#
        //for row in &piece.data {                                                                    //#
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

fn rotate_piece(piece: &mut tetris::StrippedPiece) {
    let height = piece.data.len();
    let width = piece.data[0].len();                    //UNCHECKED INDEX
    let original = piece.data.clone();
    for row in 0..height {
        for column in 0..width {
            piece.data[row][column] = original[column][width-row-1];
        }
    }
}

///checks piece for collision on board
fn check_collision(board: &tetris::StrippedData, piece: &tetris::StrippedPiece) -> bool {
    for row in 0..piece.data.len() {
        for column in 0..piece.data[row].len() {
            if piece.data[row][column] {
                if let Some(y) = board.get((piece.location.1+row as isize) as usize) {                      //RELIES ON USIZE WRAPPING
                    if let Some(x) = y.get((piece.location.0+column as isize) as usize) {                //RELIES ON USIZE WRAPPING
                        if *x {return true}
                    } else {return true}
                } else {return true}
            }
        }
    }
    false
}

///get all possible moves for a piece
fn get_moves_for_piece(board: &tetris::StrippedData, mut piece: tetris::StrippedPiece, is_held: bool, parameters: &AiParameters) -> Vec<MoveData> {
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
        rotate_piece(&mut piece);
    }
    possible_moves
}

///get all possible moves for current board
fn get_possible_moves(board: &tetris::StrippedBoard, parameters: &AiParameters) -> Vec<MoveData> {
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
fn get_input_move(board: tetris::StrippedBoard, parameters: &AiParameters) -> Vec<Move> {
    let mut possible_moves = get_possible_moves(&board, parameters);
    if !possible_moves.is_empty() {
        possible_moves.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));     //IF NAN DEFAULTS TO EQUAL
        possible_moves[0].gen_input(&board)
    } else {vec!(Move::Restart)}
}











pub struct Packet {
    board: Option<tetris::StrippedBoard>,
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
    pub fn send_board(&self, board: tetris::StrippedBoard) -> DynResult<()> {
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
        let mut ai_input = self.input.lock()?;
        *ai_input = input;
        Ok(())
    }

    /// notify the main thread that you got a board but you not generating doing any new moves
    /// only writes to buffer is buffer is empty, else does nothing.
    /// used to tell trainer function that it got the board
    fn notify_none(&self) -> Result<(), PoisonError<MutexGuard<Vec<Move>>>>{
        let mut ai_input = self.input.lock()?;
        if ai_input.is_empty() {*ai_input = vec!(Move::None)} 
        Ok(())
    }
}

///for every packet received calculates moves
fn ai_loop(radio: AiRadio, parameters: AiParameters) {
    let mut last_board = Vec::new();
    for packet in &radio.rx {
        if let Some(board) = packet.board {
            if board.data != last_board {
                last_board = board.data.clone();
                if !board.gameover {
                    check!(radio.set_input(get_input_move(board, &parameters)));
                } else {check!(radio.set_input(vec!(Move::Restart)))}
            } else {check!(radio.notify_none())}
        } else if packet.exit {break}
    }
}

///starts the AI thread
pub fn start(parameters: &AiParameters) -> MainRadio {
    //clean!("ai.log");
    let input = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel();
    let ai_radio = AiRadio {input: Arc::clone(&input), rx};
    let parameters = parameters.clone();
    let handle = thread::spawn(move || {ai_loop(ai_radio, parameters)});
    MainRadio {tx, input, handle: Some(handle)}
}
//have arg to run it with AI. maybe button in game? maybe have it compete against player?

//run in separate thread
//two way communication, rx and tx for both.
//ai sits in a for rx loop. called each time a board is passed                                      PASS RIGHT AFTER UPDATE    MAYBE USE BOARD.GET_BOARD() METHOD
//if piece == spawn location then calc board and set a list of desired input and coords             WHAT ABOUT AFTER PIECE SET WHEN PIECE SPAWNED BUT BOARD NOT UPDATED? WHEN UPDATED IT DROPS PIECE
//loop and send move_list.next()                        SET DELAY ON SENDING FOR DIFFICULTY         DECOUPLE FROM UPDATE TO ALLOW SPEED         WHAT IF IT DESYNCS A PIECE?

//on each input update check for input from ai thread

//check every rotation at every space               LAZY
//for determining move value
//  iter rows in reverse
//  get height
//  get num of completed lines
//  get holes (if block empty check block above)
//  convert to itering columns then:
//      get average height of each column then analyse variation
//      avoid gaps more than 4 tall                                                 ENCOURAGE GAPS 4 TALL?? FOR TETS
//weight of each parameter val is a -1-1 float.                                     USE GENERATIONAL ALG TO TEST ON VERSION WITH NO DELAY
//HOW TO GET MOVE VALUE?
//also check held piece. if no held piece then check next piece
//choose best move out of them all

//goal: have a function that takes a board and returns a series of input
//  parse and convert board
//  scan for input. return coords, rotation, and if hold piece
//  find best location
//  generate input to get to coords, rotation, target piece

//  input handler that gives move_list.next() when asked                DELAY HANDLED BY AI? WORRY ABOUT IT AFTER TRAINING. FOR NOW GO FOR ONE INPUT PER FRAME


//CHANGE TETRIS CODE AS LITTLE AS POSSIBLE

//GEN ALG
//  play game til gameover          ONE INPUT PER FRAME
//  use score to calc               MAYBE AIM FOR LOWER LEVELS TOO? TO ENCOURAGE TETS INSTEAD OF ONE LINE MATCHES

//CANT DO FANCY LAST SECOND input



//arc mutex to hold iter of input