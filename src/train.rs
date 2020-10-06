//! THIS ENTIRE MODULE IS A MESS! but hey...its not meant for production, only training so...it works i guess?
mod breed;
mod progress;
mod display;

use super::game::{Board, Move};
use super::ai;
use dynerr::*;

use std::fmt;
use std::time::Instant;
use std::sync::{Arc, Mutex, mpsc};

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


//10x200  0:54 var 103764
//15x200  1:19 var 78181
//20x200  1:44 var 61896
//25x200  2:08 var 57244
//50x200  4:11 var 53444
//100x200 8:38 var 38744

///times to run each AIs game
const SIM_TIMES: usize          = 50;   //50
///how many game sims can be running at once
const POOL_SIZE: usize          = 10;   //10
///generation size
const BATCH_SIZE: usize         = 200;  //200
///how many generations to run
const GENERATIONS: usize        = 0;    //IF 0 THEN INFINITE
///max level before timeout
const MAX_LEVEL: usize          = 50;   //20
///how often to update screen
const DISPLAY_INTERVAL: usize   = 13;

//range that usize parameters can be between
const U_RANGE: (usize, usize)   = (0, 4);       //max *should* be 4. inclusive upper
//range that float parameters can be between
const F_RANGE: (f32, f32)       = (0.0, 1.0);
//how big a usize nudge is
const U_NUDGE: usize            = 1;
//range that a float nudge can be between
const F_NUDGE_RANGE: (f32,f32)  = (0.001, 0.02);

const BREEDER_PERCENT: f32      = 0.20; //extra space will be filled in with randoms
const PERCENT_CROSS: f32        = 0.80; //extra space will be filled in with randoms
const PERCENT_INSERT: f32       = 0.00; //extra space will be filled in with randoms

const INSERT_CHANCE: f32        = 0.10; //% of chromosomes in insert pool will be inserted
const NUDGE_CHANCE: f32         = 0.10; //% of chromosomes will get nudged
const MUTATION_CHANCE: f32      = 0.05; //% of chromosomes will mutate






///the results of a game
#[derive(Clone, PartialEq, Copy)]
pub struct GameResult {
    score: usize,
    level: usize,
    placed: usize,
    parameters: Option<ai::AiParameters>,       //an option to cut down on clones.
}

impl GameResult {
    ///gets the average results of a set of games
    fn get_averaged(results: Vec<Self>, parameters: ai::AiParameters) -> Self {
        Self {
            score:  results.iter().map(|r| r.score).sum::<usize>()/results.len(),
            level:  results.iter().map(|r| r.level).sum::<usize>()/results.len(),
            placed: results.iter().map(|r|r.placed).sum::<usize>()/results.len(),
            parameters: Some(parameters),                                  
        }
    }

    ///prints column labels aligned to formatting
    fn print_header() {
        println!("RANK |  SCORE  | LEVEL | PLACED | PARAMS");
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{:>7} |  {:>3}  | {:>6} | {}",
            self.score,
            self.level,
            self.placed,
            if let Some(p) = &self.parameters {format!("{}", p)} else {String::new()})
    }
}






///plays a game SIM_TIMES times
fn play_game(board: Arc<Board>, parameters: ai::AiParameters, progress: Arc<Mutex<usize>>) -> GameResult {
    let mut results = Vec::new();
    let mut ai_radio = ai::start(parameters, false);
    for _ in 0..SIM_TIMES {
        let mut sim_board = (*board).clone();
        let mut placed = 0;
        while !sim_board.gameover && sim_board.level < MAX_LEVEL {
            check!(ai_radio.send_board(sim_board.get_board()));
            loop {
                if let Some(ai_input) = check!(ai_radio.get_input()) {
                    match ai_input {
                        ai::Move::Left      => {sim_board.move_piece(Move::Left);},
                        ai::Move::Right     => {sim_board.move_piece(Move::Right);},
                        ai::Move::Rotate    => {sim_board.rotate_piece();}
                        ai::Move::Drop      => {check!(sim_board.drop_piece());placed+=1;},
                        ai::Move::Hold      => {check!(sim_board.hold_piece());},
                        ai::Move::Restart   => {},
                        ai::Move::None      => {},
                    }
                    break
                }
            }
        }
        results.push(
            GameResult{
                parameters: None,
                score: sim_board.score,
                level: sim_board.level,
                placed,
            }
        );
        *(progress.lock().unwrap())+=1;
    }
    check!(ai_radio.join());
    GameResult::get_averaged(results, parameters)
}



///takes Vec<Parameters> and does generation
fn do_generation(generation: Vec<ai::AiParameters>) -> DynResult<Vec<GameResult>> {
    let master_board = Arc::new(Board::new_board()?);
    let progress = Arc::new(Mutex::new(0));
    let display_thread = display::DisplayThread::start(Arc::clone(&progress));
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
    results.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(results)
}



///does the actual training
pub fn train() -> DynResult<()> {
    let (mut gen, past_elapsed, mut generation) = match check!(progress::get_progress()) {
        Some((gen, elapsed, results)) => {
            println!("RESUMING SPECIES FROM GENERATION {}", gen);
            (gen, elapsed, breed::breed_next_gen(&results))            
        },
        None => {
            println!("STARTING NEW SPECIES");
            (0, 0, breed::new_species())
        },
    };
    let mut best_results = check!(progress::BestResult::get());
    let mut time_handle = display::TimeTracker::new(past_elapsed);
    loop {
        gen+=1;
        println!("STARTING GENERATION {}", gen);
        time_handle.start_loop();
        let results = check!(do_generation(generation));
        display::display_gen_info(&time_handle, gen-1, &results);
        progress::BestResult::update(&mut best_results, &results, gen);
        let breeders = &results[0..(BATCH_SIZE as f32*BREEDER_PERCENT) as usize];
        progress::log_stats(&best_results, breeders, gen, &time_handle);
        generation = breed::breed_next_gen(breeders);
        if gen >= GENERATIONS && GENERATIONS!=0 {break}
    }
    let total_elapsed = time_handle.total_training();
    println!("{} generations completed in {}", GENERATIONS, display::format_time(total_elapsed, "dhms"));
    println!("Average of 1 generation every {}", display::format_time(total_elapsed/gen as u64,"hms"));

    println!("BEST RESULTS");
    GameResult::print_header();
    let disp_num = {if GENERATIONS >= 10 {10} else {GENERATIONS}};
    for i in 0..disp_num {println!("  {:>2} | {}", i+1, best_results[i])}
    Ok(())
}

//cargo run --release -- --train


//10x10x100 dry run
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
//got rid of unnecessary clones in tetris::Board                                            00h05m40s    |    2.94g/s    |    154p/s    |    score variation: 94906
//condensed functions inside game::pieces                                                   00h05m38s    |    2.96g/s    |    190p/s    |    score variation: 85236
//removed shadow updates for down and drop                                                  00h02m54s    |    5.75g/s    |    307p/s    |    score variation: 133120
//added sprite index                                                                        00h00m57s    |    17.54g/s    |    154p/s    |    score variation: 88490
//got rid of unnecessary copies in movement and piece swapping                              00h00m24s    |    41.67g/s    |     43p/s    |    score variation: 81104
//flattened board and pieces to 1d vec                                                      00h00m21s       |    47.62g/s    |    262    |    score variation: 350010
//tried to trim a few more clones in ai                                                     00h00m22s       |    45.45g/s    |    298    |    score variation: 334920
//CHANGING BENCHMARK TO 25X10X200                                                           00h01m50s       |    45.45g/s    |    307    |    score variation: 278325
//changed nothing???                                                                        00h01m26s       |    58.14g/s    |    243    |    score variation: 255895



//TO OPTIMIZE
//benchmark AI functions
//change pieces to vec<&bool> then shuffle references to rotate. avoids the clone required during rotation