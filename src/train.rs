use super::tetris::{Board, Move};
use super::ai;
use dynerr::*;

use std::{fmt, thread};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, mpsc};

use rand::Rng;
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


//TIMINGS
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


//VARIATION (REDUCED BATCH SIZE BUT MORES SIM STEPS BATCH_SIZE*SIM_SIZE == 1000) 
//  100x10      00h08m25s    |    1.98g/s    |    181p/s    |    score variation: 67722
//  50x20       00h08m38s    |    1.93g/s    |    182p/s    |    score variation: 449570
//  40x25       00h07m49s    |    2.13g/s    |    169p/s    |    score variation: 194249

//  50x10       00h04m07s    |    2.02g/s    |    171p/s    |    score variation: 521488

//INCREASING PRIORITY HAD NO EFFECT

///times to run each AIs game
const SIM_TIMES: usize      = 10;
///how many game sims can be running at once
const POOL_SIZE: usize      = 10;
///generation size
const BATCH_SIZE: usize     = 100;
///max level before timeout
const MAX_LEVEL: usize      = 100;
///how often to update screen
const DISPLAY_INTERVAL: f32 = 12.3; 





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
            let mut last_progress = 0;
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
                    {let p = (progress-last_progress) as f32/DISPLAY_INTERVAL; if p.is_nan() {0.0} else {p}}, 
                    eta/3600, (eta%3600)/60, eta%60
                );
                last_progress = progress;
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



///the results of a game
struct GameResult {
    score: usize,
    level: usize,
    placed: usize,
    speed: usize,
    parameters: Option<ai::AiParameters>,       //an option to cut down on clones.
}

impl GameResult {
    ///gets the average results of a set of games
    fn get_averaged(results: Vec<Self>, parameters: ai::AiParameters) -> Self {
        Self {
            score:  results.iter().map(|r| r.score).sum::<usize>()/results.len(),
            level:  results.iter().map(|r| r.level).sum::<usize>()/results.len(),
            placed: results.iter().map(|r|r.placed).sum::<usize>()/results.len(),
            speed:  results.iter().map(|r| r.speed).sum::<usize>()/results.len(),
            parameters: Some(parameters),                                                              //EXPLICIT UNWRAP ON POP
        }
    }

    ///prints column labels aligned to formatting
    fn print_header() {
        println!("RANK |  SCORE  | LEVEL | PLACED |  SPEED | PARAMS");
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
            " {:>7} |  {:>3}  | {:>6} | {:>3?}p/s | {}", 
            self.score, 
            self.level, 
            self.placed, 
            self.speed,  
            if let Some(p) = &self.parameters {format!("{}", p)} else {String::new()})
    }
}


///does the actual training
pub fn train() -> DynResult<()> {
    //let generation = (0..BATCH_SIZE).map(|_| {
    //    ai::AiParameters {
    //        min_cleared_rows:               3,
    //        cleared_rows_importance:        0.50,
    //        piece_depth_importance:         0.25,
    //        max_height_importance:          0.75,
    //        avg_height_importance:          0.0,
    //        height_variation_importance:    0.5,
    //        current_holes_importance:       3.5,
    //        max_pillar_height:              2,
    //        current_pillars_importance:     0.75,
    //    }
    //}).collect::<Vec<ai::AiParameters>>();
    let mut rng = rand::thread_rng();
    let generation = (0..BATCH_SIZE).map(|_| {
        ai::AiParameters {
            min_cleared_rows:               rng.gen_range(0,5),
            cleared_rows_importance:        rng.gen_range(0.0,1.0),
            piece_depth_importance:         rng.gen_range(0.0,1.0),
            max_height_importance:          rng.gen_range(0.0,1.0),
            avg_height_importance:          rng.gen_range(0.0,1.0),
            height_variation_importance:    rng.gen_range(0.0,1.0),
            current_holes_importance:       rng.gen_range(0.0,1.0),
            max_pillar_height:              rng.gen_range(0,5),
            current_pillars_importance:     rng.gen_range(0.0,1.0),
        }
    }).collect::<Vec<ai::AiParameters>>();


    let start = Instant::now();
    let results = check!(do_generation(generation));
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

///takes Vec<Parameters> and does generation
fn do_generation(generation: Vec<ai::AiParameters>) -> DynResult<Vec<GameResult>> {
    let board = Board::new_board()?;
    let progress = Arc::new(Mutex::new(0));
    let display_thread = DisplayThread::start(Arc::clone(&progress));
    let (tx, rx) = mpsc::channel();

    let pool = ThreadPool::new(POOL_SIZE);
    for child in generation {
        let board_clone = board.clone();
        let progress = Arc::clone(&progress);
        let tx = tx.clone();
        pool.execute(move || check!(tx.send(play_game(board_clone, child, progress))));
    }
    pool.join();

    let mut results = rx.iter().take(BATCH_SIZE).collect::<Vec<GameResult>>();
    display_thread.stop()?;
    results.sort_by(|a, b| a.score.cmp(&b.score));
    results.reverse();
    Ok(results)
}


///plays a game SIM_TIMES times
fn play_game(mut board: Board, parameters: ai::AiParameters, progress: Arc<Mutex<usize>>) -> GameResult {
    let mut results = Vec::new();
    let mut ai_radio = ai::start(&parameters);
    for _ in 0..SIM_TIMES {
        let mut placed = 0;
        let start = Instant::now();
        while !board.gameover && board.level < MAX_LEVEL {
            check!(ai_radio.send_board(board.get_board()));
            loop {
                if let Some(ai_input) = check!(ai_radio.get_input()) {
                    match ai_input {
                        ai::Move::Left      => {board.move_piece(Move::Left);},
                        ai::Move::Right     => {board.move_piece(Move::Right);},
                        ai::Move::Rotate    => {board.move_piece(Move::Rotate);}
                        ai::Move::Drop      => {board.move_piece(Move::Drop);placed+=1;},
                        ai::Move::Hold      => {check!(board.piece_hold());},
                        ai::Move::Restart   => {},
                        ai::Move::None      => {},
                    }
                    break
                }
            }
            check!(board.try_update());
        }
        results.push(
            GameResult{
                parameters: None,
                score: board.score, 
                level: board.level,
                placed, 
                speed: placed.checked_div((Instant::now()-start).as_secs() as usize).unwrap_or(0)
            }
        );
        check!(board.reset());
        let mut prog = progress.lock().unwrap();
        *prog+=1;
    }    
    check!(ai_radio.join());
    GameResult::get_averaged(results, parameters)
}

//cargo run --release -- --train












//gen random starting values between -1 and 1
//game
//take top 10%
//select random pairs and create children
//30% averaged, 30% random point between two params, 30% swap random amount of values between the two, 10% original chosen
//mutate X% within MUTATE_CHANCE_RANGE of them by MUTATE_SEVERITY_RANGE in POS_NEG_FLAG direction
//
//maybe have "master list" of the top 5% of every generation then average that to get a master piece?




//issue: maxing out move counter with insanely low score by only going after single pieces
//solutions:
//  maybe have the limit be levels instead of turns? that way it avoids level spammers
//  bring back some version of the score/level thing

//measuring off purely score with max move timeout causes level spammer issue
//GENERATION COMPLETED IN 00h06m35s    |    2.53g/s    |    117p/s    |    score variation: 994856
//RANK |  SCORE  | LEVEL | PLACED |  SPEED | PARAMS
//   1 |  995780 |  399  |  10001 | 222p/s | 0 : 0.873 : 0.581 : 0.134 : 0.324 : 0.671 : 0.901 : 4 : 0.607
//   2 |  957420 |  399  |  10001 | 175p/s | 1 : 0.283 : 0.254 : 0.127 : 0.571 : 0.227 : 0.889 : 0 : 0.930
//   3 |  920360 |  267  |   6731 | 214p/s | 2 : 0.384 : 0.071 : 0.647 : 0.939 : 0.480 : 0.983 : 2 : 0.511
//   4 |  893260 |  399  |  10001 | 142p/s | 0 : 0.956 : 0.860 : 0.130 : 0.616 : 0.306 : 0.454 : 3 : 0.784
//   5 |  878680 |  399  |  10001 | 192p/s | 4 : 0.075 : 0.547 : 0.139 : 0.747 : 0.211 : 0.412 : 1 : 0.222


//50 level time out causes gameover to not be as penalized
//GENERATION COMPLETED IN 00h05m09s    |    3.24g/s    |    132p/s    |    score variation: 273564
//RANK |  SCORE  | LEVEL | PLACED |  SPEED | PARAMS
//   1 |  274290 |   22  |    605 | 261p/s | 3 : 0.542 : 0.267 : 0.498 : 0.184 : 0.159 : 0.914 : 3 : 0.489
//   2 |  171326 |   38  |    996 | 246p/s | 3 : 0.162 : 0.867 : 0.122 : 0.806 : 0.306 : 0.898 : 4 : 0.195
//   3 |  163692 |   50  |   1267 | 151p/s | 0 : 0.434 : 0.924 : 0.243 : 0.970 : 0.081 : 0.703 : 4 : 0.122
//   4 |  152206 |   48  |   1236 | 158p/s | 1 : 0.569 : 0.583 : 0.692 : 0.033 : 0.406 : 0.928 : 3 : 0.035
//   5 |  148418 |   11  |    326 | 215p/s | 3 : 0.912 : 0.669 : 0.296 : 0.893 : 0.105 : 0.945 : 2 : 0.561

//i think having a 100 level timeout and rewarding based off score will work now that input is limited
//now i just need to optimize and impl the actual evolutionary alg