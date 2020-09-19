use super::tetris::{Board, Move};
use super::ai;
use dynerr::*;

use std::{fmt, thread};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, mpsc};

use threadpool::ThreadPool;
//my params
//let parameters = ai::AiParameters {
//    min_cleared_rows:               3,
//    cleared_rows_importance:        0.50,
//    piece_depth_importance:         0.25,
//    max_height_importance:          0.75,
//    avg_height_importance:          0.0,
//    height_variation_importance:    0.5,
//    current_holes_importance:       3.5,
//    max_pillar_height:              2,
//    current_pillars_importance:     0.75,
//};



//batch times
//  100 = 78s   
//  50  = 38s   (x2 = 76s)
//  25  = 14s   (x4 = 56s)

//pooled
//  25x4 = 68s
//  20x5 = 64s
//  10x10= 82s

//multi sim pooled
//  10x20x5     1084s avg 7.97ms    var 126682-501204
//  10x100x1    561s  avg 41.8ms    var 100672-451348

//proper pooled
//  10x20x100   00h08m11s    |    2.04g/s    |     93p/s    |    score variation: 374846
//  10x15x100   00h08m31s    |    1.96g/s    |    119p/s    |    score variation: 419320
//  10x12x100   00h08m04s    |    2.07g/s    |    149p/s    |    score variation: 476724
//  10x11x100   00h08m03s    |    2.07g/s    |    162p/s    |    score variation: 424602
//  10x10x100   00h08m02s    |    2.07g/s    |    174p/s    |    score variation: 384238
//  10x10x100   00h08m14s    |    2.02g/s    |    171p/s    |    score variation: 371070
//  10x9x100    00h08m39s    |    1.93g/s    |    184p/s    |    score variation: 433620
//  10x7x100    00h08m37s    |    1.93g/s    |    219p/s    |    score variation: 423344
//  10x5x100    00h09m50s    |    1.69g/s    |    249p/s    |    score variation: 429140
//  10x3x100    00h14m09s    |    1.18g/s    |    295p/s    |    score variation: 486018
//  10x2x100    00h02m03s    |     10%    |    0.81game/s    |    ETA: 00h18m27s
//  10x1x100    00h02m03s    |     06%    |    0.50game/s    |    ETA: 00h31m00s

// 100x10x10    00h08m25s    |    1.98g/s    |    181p/s    |    score variation: 67722

///times to run each AIs game
const SIM_TIMES: usize      = 10;
///how many game sims can be running at once
const POOL_SIZE: usize      = 10;
///generation size
const BATCH_SIZE: usize     = 100;
///max pieces a game can set before ending
const MAX_MOVES: usize      = 10000;
///how often to update screen
const DISPLAY_INTERVAL: f32 = 12.3; 


///the results of a game
struct GameResult {
    score: usize,
    level: usize,
    placed: usize,
    speed: usize,
    parameters: ai::AiParameters,
}

impl GameResult {
    ///gets the average results of a set of games
    fn get_averaged(mut results: Vec<Self>) -> Self {
        Self {
            score:  results.iter().map(|r| r.score).sum::<usize>()/results.len(),
            level:  results.iter().map(|r| r.level).sum::<usize>()/results.len(),
            placed: results.iter().map(|r| r.placed).sum::<usize>()/results.len(),
            speed:  results.iter().map(|r| r.speed).sum::<usize>()/results.len(),
            parameters: results.pop().unwrap().parameters,                                                              //EXPLICIT UNWRAP ON POP
        }
    }

    ///prints column labels aligned to formatting
    fn print_header() {
        println!("RANK |  SCORE  | LEVEL | PLACED |  SPEED | PARAMS");
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {:>7} |  {:>3}  | {:>6} | {:>3?}p/s | {}", self.score, self.level, self.placed, self.speed, self.parameters)
    }
}


///does the actual training
pub fn train() -> DynResult<()> {
    let parameters = ai::AiParameters {
        min_cleared_rows:               3,
        cleared_rows_importance:        0.50,
        piece_depth_importance:         0.25,
        max_height_importance:          0.75,
        avg_height_importance:          0.0,
        height_variation_importance:    0.5,
        current_holes_importance:       3.5,
        max_pillar_height:              2,
        current_pillars_importance:     0.75,
    };


    let start = Instant::now();
    let results = check!(do_generation(parameters));
    let elapsed = (Instant::now()-start).as_secs();
    
    println!("GENERATION COMPLETED IN {:02}h{:02}m{:02}s    |    {:0.2}g/s    |    {:>3}p/s    |    score variation: {}",
        elapsed/3600, (elapsed%3600)/60, elapsed%60,
        (SIM_TIMES*BATCH_SIZE) as f32/elapsed as f32,
        results.iter().map(|r| r.speed).sum::<usize>()/BATCH_SIZE,
        results[0].score-results[BATCH_SIZE-1].score
    );
    GameResult::print_header();
    for i in 0..5 {
        println!("  {:>2} |{}", i+1, results[i]);
    }

    Ok(())
}

//only takes one param now for testing
///takes Vec<Parameters> and does generation
fn do_generation(parameters: ai::AiParameters) -> DynResult<Vec<GameResult>> {
    let board = Board::new_board()?;
    let progress = Arc::new(Mutex::new(0));
    let display_thread = DisplayThread::start(Arc::clone(&progress));
    let (tx, rx) = mpsc::channel();

    let pool = ThreadPool::new(POOL_SIZE);
    for _ in 0..BATCH_SIZE {
        let board_clone = board.clone();
        let parameters = parameters.clone();
        let progress = Arc::clone(&progress);
        let tx = tx.clone();
        pool.execute(move || check!(tx.send(play_game(board_clone, parameters, progress))));
    }
    pool.join();

    let mut results = rx.iter().take(BATCH_SIZE).collect::<Vec<GameResult>>();
    display_thread.stop()?;
    results.sort_by(|a, b| a.score.cmp(&b.score));
    results.reverse();
    Ok(results)
}

///handle to the thread doing updates
struct DisplayThread {
    tx: mpsc::Sender<bool>,
    handle: thread::JoinHandle<()>,
}

impl DisplayThread {
    ///starts a display thread that prints progress statistics every x seconds
    fn start(prog: Arc<Mutex<usize>>) -> Self {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            loop {
                let elapsed = (Instant::now()-start).as_secs();
                let progress = {let p = prog.lock().unwrap(); *p};
                let percent = ((progress as f32/(SIM_TIMES*BATCH_SIZE) as f32)*100.0) as usize;
                let eta = (((SIM_TIMES*BATCH_SIZE) as u64 * elapsed)
                    .checked_div(progress as u64)
                    .unwrap_or(0))
                    .checked_sub(elapsed)
                    .unwrap_or(0);
                println!(
                    "{:02}h{:02}m{:02}s    |    {:>4}    |    {:.02}g/s    |    ETA: {:02}h{:02}m{:02}s", 
                    elapsed/3600, (elapsed%3600)/60, elapsed%60,
                    format!("{:02}%",percent), 
                    {let p = progress as f32/elapsed as f32; if p.is_nan() {0.0} else {p}}, 
                    eta/3600, (eta%3600)/60, eta%60
                );
                if let Ok(true) = rx.try_recv() {break} 
                else {thread::sleep(Duration::from_secs_f32(DISPLAY_INTERVAL))}
            }
        });
        Self {tx, handle}
    }

    ///tell thread to exit and call join
    fn stop(self) -> DynResult<()> {
        self.tx.send(true)?;
        self.handle.join().unwrap();
        Ok(())
    }
}


///plays a game SIM_TIMES times
fn play_game(mut board: Board, parameters: ai::AiParameters, progress: Arc<Mutex<usize>>) -> GameResult {
    let mut results = Vec::new();
    let mut ai_radio = ai::start(&parameters);
    for _ in 0..SIM_TIMES {
        let mut placed = 0;
        let start = Instant::now();
        while !board.gameover {
            if check!(board.try_update()) {
                check!(ai_radio.send_board(board.get_board()));     
                for ai_input in check!(ai_radio.wait_for_dump_input()) {
                    match ai_input {
                        ai::Move::Left      => {board.move_piece(Move::Left);},
                        ai::Move::Right     => {board.move_piece(Move::Right);},
                        ai::Move::Rotate    => {board.move_piece(Move::Rotate);}
                        ai::Move::Drop      => {board.move_piece(Move::Drop);},
                        ai::Move::Hold      => {check!(board.piece_hold());},
                        ai::Move::Restart   => {}
                    }
                }
                placed+=1;
                if placed > MAX_MOVES {break}
            }
        }
        let result = board.get_board();
        results. push(
            GameResult{
                parameters: parameters.clone(),
                score: result.score, 
                level: result.level,
                placed, 
                speed: placed.checked_div((Instant::now()-start).as_secs() as usize).unwrap_or(0)
            }
        );
        check!(board.reset());
        let mut prog = progress.lock().unwrap();
        *prog+=1;
    }    
    check!(ai_radio.join());
    GameResult::get_averaged(results)
}

//cargo run --release -- --train