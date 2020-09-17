use crate::tetris;

use dynerr::*;

use std::thread;
use std::sync::{Arc, Mutex, mpsc, PoisonError, MutexGuard};

pub struct Packet {
    board: Option<tetris::BoardData>,
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
    pub fn send_board(&self, board: tetris::BoardData) -> DynResult<()> {
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
}




///possible piece movements
pub enum Move {
    Down,
    Left,
    Right,
    Rotate,
    Drop,
    Hold,
    Restart
}
///current piece rotation relative to its start
enum Rotation {
    North,
    South,
    East,
    West,
}

use rand::Rng;
///TEMP gens random moves
fn random(len: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    for _ in 0..len {
        moves.push(
            match rand::thread_rng().gen_range(0, 6) {
                0 => Move::Down,
                1 => Move::Left,
                2 => Move::Right,
                3 => Move::Rotate,
                4 => Move::Drop,
                5 => Move::Hold,
                6 => Move::Restart,
                i => panic!("bad num {}", i),
            }
        )
    }
    moves
}

///for every packet received calculates moves
fn ai_loop(radio: AiRadio) {
    for packet in &radio.rx {
        if let Some(board) = packet.board {
            if !board.gameover {
                check!(radio.set_input(random(10)));
            } else {check!(radio.set_input(vec!(Move::Restart)))}
        } else if packet.exit {break}
    }
}

///starts the AI thread
pub fn start() -> MainRadio {
    let input = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel();
    let ai_radio = AiRadio {input: Arc::clone(&input), rx};
    let handle = thread::spawn(move || {ai_loop(ai_radio)});
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