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
//    points_scored_importance:        0.50,
//    piece_depth_importance:         0.25,
//    max_height_importance:          0.75,
//    avg_height_importance:          0.0,
//    height_variation_importance:    0.5,
//    current_holes_importance:       3.5,
//    max_pillar_height:              2,
//    current_pillars_importance:     0.75,
//};



///times to run each AIs game
const SIM_TIMES: usize      = 10;
///how many game sims can be running at once
const POOL_SIZE: usize      = 10;
///generation size
const BATCH_SIZE: usize     = 100;
///max level before timeout
const MAX_LEVEL: usize      = 100;
///how often to update screen
const DISPLAY_INTERVAL: usize = 13; 





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
                    "{:02}h{:02}m{:02}s    |    {:>3}    |    {:.02}g/s    |    ETA: {:02}h{:02}m{:02}s", 
                    elapsed/3600, (elapsed%3600)/60, elapsed%60,
                    format!("{:02}%",percent), 
                    {let p = (progress-last_progress) as f32/DISPLAY_INTERVAL as f32; if p.is_nan() {0.0} else {p}}, 
                    eta/3600, (eta%3600)/60, eta%60
                );
                last_progress = progress;
                for _ in 0..DISPLAY_INTERVAL {
                    if let Ok(true) = rx.try_recv() {return} 
                    thread::sleep(Duration::from_secs(1))
                }
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
pub fn train(dry_run: bool) -> DynResult<()> {
    let generation = {
        if dry_run {
            (0..BATCH_SIZE).map(|_| {
                ai::AiParameters {
                    points_scored_importance:       0.50,
                    piece_depth_importance:         0.25,
                    max_height_importance:          0.75,
                    avg_height_importance:          0.0,
                    height_variation_importance:    0.5,
                    current_holes_importance:       3.5,
                    max_pillar_height:              2,
                    current_pillars_importance:     0.75,
                }
            }).collect::<Vec<ai::AiParameters>>()
        } else {
            let mut rng = rand::thread_rng();
            (0..BATCH_SIZE).map(|_| {
                ai::AiParameters {
                    points_scored_importance:       rng.gen_range(0.0,1.0),
                    piece_depth_importance:         rng.gen_range(0.0,1.0),
                    max_height_importance:          rng.gen_range(0.0,1.0),
                    avg_height_importance:          rng.gen_range(0.0,1.0),
                    height_variation_importance:    rng.gen_range(0.0,1.0),
                    current_holes_importance:       rng.gen_range(0.0,1.0),
                    max_pillar_height:              rng.gen_range(0,5),
                    current_pillars_importance:     rng.gen_range(0.0,1.0),
                }
            }).collect::<Vec<ai::AiParameters>>()
        }
    };
    

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
    let master_board = Arc::new(Board::new_board()?);
    let progress = Arc::new(Mutex::new(0));
    let display_thread = DisplayThread::start(Arc::clone(&progress));
    let (tx, rx) = mpsc::channel();

    let pool = ThreadPool::new(POOL_SIZE);
    for child in generation {
        let board_ref = Arc::clone(&master_board);
        let progress = Arc::clone(&progress);
        let tx = tx.clone();
        pool.execute(move || check!(tx.send(play_game(board_ref, child, progress))));
    }
    pool.join();

    let mut results = rx.iter().take(BATCH_SIZE).collect::<Vec<GameResult>>();
    display_thread.stop()?;
    results.sort_by(|a, b| a.score.cmp(&b.score));
    results.reverse();
    Ok(results)
}


///plays a game SIM_TIMES times
fn play_game(board: Arc<Board>, parameters: ai::AiParameters, progress: Arc<Mutex<usize>>) -> GameResult {
    let mut results = Vec::new();
    let mut ai_radio = ai::start(parameters.clone(), false);
    for _ in 0..SIM_TIMES {
        let mut sim_board = (*board).clone();
        let mut placed = 0;
        let start = Instant::now();
        while !sim_board.gameover && sim_board.level < MAX_LEVEL {
            check!(ai_radio.send_board(sim_board.get_board()));
            loop {
                if let Some(ai_input) = check!(ai_radio.get_input()) {
                    match ai_input {
                        ai::Move::Left      => {sim_board.move_piece(Move::Left);},
                        ai::Move::Right     => {sim_board.move_piece(Move::Right);},
                        ai::Move::Rotate    => {sim_board.move_piece(Move::Rotate);}
                        ai::Move::Drop      => {sim_board.move_piece(Move::Drop);placed+=1;},
                        ai::Move::Hold      => {check!(sim_board.piece_hold());},
                        ai::Move::Restart   => {},
                        ai::Move::None      => {},
                    }
                    break
                }
            }
            check!(sim_board.try_update());
        }
        results.push(
            GameResult{
                parameters: None,
                score: sim_board.score, 
                level: sim_board.level,
                placed, 
                speed: placed.checked_div((Instant::now()-start).as_secs() as usize).unwrap_or(0)
            }
        );
        *(progress.lock().unwrap())+=1;
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



//before pooling it was at 8m30s
//after its at 7m30s
//benchmarks using my params 10x10x100
//START                                                                                     00h07m13s    |    2.31g/s    |    147p/s    |    score variation: 193450
//changed master board to Arc<T> and removed board.reset() in sims                          00h06m35s    |    2.53g/s    |    154p/s    |    score variation: 164542
//adjusted locks in ai Arc<Mutex<T>>                                                        00h06m11s    |    2.70g/s    |    156p/s    |    score variation: 195230
//stopped DisplayThread from blocking during sleep so join() happens faster after pool exit 00h06m15s    |    2.67g/s    |    158p/s    |    score variation: 187122
//converted from 2d vec to 1d, changed "lines cleared" to "points scored", combined all height functions            MAY AFFECT TIMES
//                                                                                          00h15m41s    |    1.06g/s    |    174p/s    |    score variation: 108042
//switched y*width+x to indexing use idx = x; idx += self.width                             00h06m11s    |    2.70g/s    |    169p/s    |    score variation: 78930
//changed indexing to use constants from tetris                                             00h06m52s    |    2.43g/s    |    162p/s    |    score variation: 92050
//                                                                                          00h07m03s    |    2.36g/s    |    173p/s    |    score variation: 113170
//REVERTED CONSTANTS FOR INDEXING                                                           00h06m30s    |    2.56g/s    |    169p/s    |    score variation: 80768