use super::game::{Board, Move};
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


//10x200  0:54 var 103764
//15x200  1:19 var 78181
//20x200  1:44 var 61896
//25x200  2:08 var 57244
//50x200  4:11 var 53444
//100x200 8:38 var 38744

///times to run each AIs game
const SIM_TIMES: usize          = 25;   //25
///how many game sims can be running at once
const POOL_SIZE: usize          = 10;   //10
///generation size
const BATCH_SIZE: usize         = 200;  //200
///how many generations to run
const GENERATIONS: usize        = 10;   //200
///max level before timeout
const MAX_LEVEL: usize          = 50;
///how often to update screen
const DISPLAY_INTERVAL: usize   = 13;

const MUTATION_CHANCE: f32      = 00.3; //of chromosomes will mutate


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
                for _ in 0..DISPLAY_INTERVAL {
                    if let Ok(true) = rx.try_recv() {return}
                    thread::sleep(Duration::from_secs(1))
                }
                let elapsed = (Instant::now()-start).as_secs();
                let progress = {let p = prog.lock().unwrap(); *p};
                let percent = ((progress as f32/(SIM_TIMES*BATCH_SIZE) as f32)*100.0) as usize;
                let eta = (((SIM_TIMES*BATCH_SIZE) as u64 * elapsed)
                    .checked_div(progress as u64)
                    .unwrap_or(0))
                    .checked_sub(elapsed)
                    .unwrap_or(0);
                println!(
                    "{}    |   {:>4}    |    {:5.02}g/s    |    ETA: {}",
                    format_time(elapsed, "hms"),
                    format!("{:02}%",percent),
                    {let p = (progress-last_progress) as f32/DISPLAY_INTERVAL as f32; if p.is_nan() {0.0} else {p}},
                    format_time(eta, "hms")
                );
                last_progress = progress;
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
#[derive(Clone)]
struct GameResult {
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
            parameters: Some(parameters),                                                              //EXPLICIT UNWRAP ON POP
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
            " {:>7} |  {:>3}  | {:>6} | {}",
            self.score,
            self.level,
            self.placed,
            if let Some(p) = &self.parameters {format!("{}", p)} else {String::new()})
    }
}

fn format_time(seconds: u64, formatting: &str) -> String {
    let mut disp = String::new();
    for c in formatting.chars() {
        disp.push_str(
            &format!(
                "{:02}{}",
                match c {
                    'd' => seconds/86400,
                    'h' => (seconds%86400)/3600,
                    'm' => (seconds%3600)/60,
                    's' => seconds%60,
                    i   => panic!("Cant format time value {}",i)
                },
                c
            )
        )
    }
    disp
}

fn display_gen_info(total_start: Instant, start: Instant, gen: usize, results: &Vec<GameResult>) {
    let elapsed = (Instant::now()-start).as_secs();
    println!("GENERATION {} COMPLETED IN {}       |    {:0.2}g/s    |    {:>3}    |    score variation: {}",
        gen+1,
        format_time(elapsed, "hms"),
        (SIM_TIMES*BATCH_SIZE) as f32/elapsed as f32,
        results.iter().map(|r| r.placed).sum::<usize>()/results.len(),
        results[0].score-results[BATCH_SIZE-1].score
    );
    let total_elapsed = (Instant::now()-total_start).as_secs();
    let eta = ((GENERATIONS as u64 * total_elapsed)
                .checked_div(gen as u64)
                .unwrap_or(0))
                .checked_sub(total_elapsed)
                .unwrap_or(0);
    println!("ELAPSED: {}           |           TOTAL ETA: {}",
        format_time(total_elapsed, "dhms"),
        format_time(eta, "dhms"),
    );
    let disp_num = {if BATCH_SIZE >= 10 {10} else {BATCH_SIZE}};
    println!("Ao{}   {:>7} |   {:>2}",
        disp_num,
        results[0..disp_num].iter().map(|r| r.score).sum::<usize>()/disp_num,
        results[0..disp_num].iter().map(|r| r.level).sum::<usize>()/disp_num,
    );
    GameResult::print_header();
    let disp_num = {if BATCH_SIZE >= 3 {3} else {BATCH_SIZE}};
    for i in 0..disp_num {
        println!("  {:>2} |{}", i+1, results[i]);
    }
    println!("\n");
}



///plays a game SIM_TIMES times
fn play_game(board: Arc<Board>, parameters: ai::AiParameters, progress: Arc<Mutex<usize>>) -> GameResult {
    let mut results = Vec::new();
    let mut ai_radio = ai::start(parameters.clone(), false);
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
            check!(sim_board.try_update());
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
    results.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(results)
}

macro_rules! U {
    ($x:expr) => {
        {if let Self::U(i) = $x {i} else {panic!("Invalid Param")}}
    };
}

macro_rules! F {
    ($x:expr) => {
        {if let Self::F(i) = $x {i} else {panic!("Invalid Param")}}
    };
}

#[derive(Clone, Copy, Debug)]
enum Params {
    F(f32),
    U(usize),
}

impl Params {
    fn get(params: &ai::AiParameters) -> [Self;10] {
        [
            Self::U(params.min_lines_to_clear),
            Self::F(params.lines_cleared_importance),
            Self::F(params.points_scored_importance),
            Self::F(params.piece_depth_importance),
            Self::F(params.max_height_importance),
            Self::F(params.avg_height_importance),
            Self::F(params.height_variation_importance),
            Self::F(params.current_holes_importance),
            Self::U(params.max_pillar_height),
            Self::F(params.current_pillars_importance),
        ]
    }

    fn construct(params: [Self;10]) -> ai::AiParameters {
        ai::AiParameters {
            min_lines_to_clear:             U!(params[0]),
            lines_cleared_importance:       F!(params[1]),
            points_scored_importance:       F!(params[2]),
            piece_depth_importance:         F!(params[3]),
            max_height_importance:          F!(params[4]),
            avg_height_importance:          F!(params[5]),
            height_variation_importance:    F!(params[6]),
            current_holes_importance:       F!(params[7]),
            max_pillar_height:              U!(params[8]),
            current_pillars_importance:     F!(params[9]),
        }
    }
}


fn breed_next_gen(breeders: &[GameResult]) -> Vec<ai::AiParameters> {
    let mut rng = rand::thread_rng();
    let mut kids = Vec::with_capacity(BATCH_SIZE);
    let params = breeders.iter().map(|b|
        Params::get(&b.parameters.as_ref().unwrap())
    ).collect::<Vec<[Params;10]>>();

    let get_couple = |mut rng: rand::prelude::ThreadRng| {
        let couple = {
            let (mut x, mut y) = (rng.gen_range(0,params.len()), rng.gen_range(0,params.len()));
            while x==y {
                x = rng.gen_range(0,params.len());
                y = rng.gen_range(0,params.len());
            }
            (x,y)
        };
        let male = &params[couple.0];
        let fema = &params[couple.1];
        (male, fema)
    };

    //left and right half's
    for _ in 0..(BATCH_SIZE as f32*0.30) as usize {
        let (male, fema) = get_couple(rng);
        let mut kid = [Params::U(0);10];
        let divide = rng.gen_range(0, kid.len());
        for x in 0..divide {kid[x] = male[x]}
        for y in divide..kid.len() {kid[y] = fema[y]}
        kids.push(kid);
    }

    //random fields
    for _ in 0..(BATCH_SIZE as f32*0.30) as usize {
        let (male, fema) = get_couple(rng);
        let mut kid = [Params::U(0);10];
        for x in 0..kid.len() {
            match rng.gen_range(0,2) {
                0 => kid[x] = male[x],
                _ => kid[x] = fema[x],
            }
        }
        kids.push(kid);
    }

    //averages
    for _ in 0..(BATCH_SIZE as f32*0.30) as usize {
        let (male, fema) = get_couple(rng);
        let mut kid = [Params::U(0);10];
        for x in 0..kid.len() {
            if let Params::U(mu) = male[x] {
                if let Params::U(fu) = fema[x] {
                    kid[x] = Params::U((mu+fu)/2)
                }
            }
            else if let Params::F(mf) = male[x] {
                if let Params::F(ff) = fema[x] {
                    kid[x] = Params::F((mf+ff)/2.0)
                }
            }
        }
        kids.push(kid);
    }

    //mutate
    for cronenberg in &mut kids {
        if rng.gen_range(0.0,100.0) < MUTATION_CHANCE*cronenberg.len() as f32 {
            match &mut cronenberg[rng.gen_range(0,cronenberg.len())] {
                Params::U(u) => *u = rng.gen_range(0,5),
                Params::F(f) => *f = rng.gen_range(0.0,1.0),
            }
        }
    }
    kids.extend(params);
    assert_eq!(kids.len(), BATCH_SIZE);
    kids.iter().map(|k| Params::construct(*k)).collect()
}



///does the actual training
pub fn train(dry_run: bool) -> DynResult<()> {
    if dry_run {println!("Starting Dry Run")}
    else {println!("Starting Live Run")}
    let mut best_results = Vec::new();
    let total_start = Instant::now();
    let mut generation = {
        if dry_run {
            (0..BATCH_SIZE).map(|_| {
                ai::AiParameters {
                    min_lines_to_clear:             3,
                    lines_cleared_importance:       0.50,
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
                    min_lines_to_clear:             rng.gen_range(0,5),
                    lines_cleared_importance:       rng.gen_range(0.0,1.0),
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
    for gen in 0..GENERATIONS {
        println!("STARTING GENERATION {}", gen+1);
        let start = Instant::now();
        let results = check!(do_generation(generation));
        display_gen_info(total_start, start, gen, &results);
        best_results.extend_from_slice(&results[0..{if BATCH_SIZE >= 3 {3} else {BATCH_SIZE}}]);
        generation = breed_next_gen(&results[0..BATCH_SIZE/10]);
    }
    let total_elapsed = Instant::now()-total_start;
    println!("{} generations completed in {}", GENERATIONS, format_time(total_elapsed.as_secs(), "dhms"));
    println!("Average of 1 generation every {}", format_time((total_elapsed/GENERATIONS as u32).as_secs(),"hms"));

    best_results.sort_by(|a, b| b.score.cmp(&a.score));
    println!("BEST RESULTS");
    GameResult::print_header();
    let disp_num = {if GENERATIONS >= 10 {10} else {GENERATIONS}};
    for i in 0..disp_num {
        println!("  {:>2} |{}", i+1, best_results[i]);
    }
    Ok(())
}

//cargo run --release -- --train --dry



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



//10x10x100
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

//850~ placed per game
//5 moves~ per place + drop
//4250 moves ver game
//update after each move



//TO OPTIMIZE
//benchmark AI functions
//WHY IS FIRST GEN FAST AND EVERY OTHER GEN 50% SLOWER?