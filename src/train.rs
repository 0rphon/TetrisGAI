//! THIS ENTIRE MODULE IS A MESS! but hey...its not meant for production, only training so...it works i guess?
use std::io::ErrorKind::NotFound;
use super::game::{Board, Move};
use super::ai;
use dynerr::*;

use std::{fmt, thread};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, mpsc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::Split;
use std::mem;

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
const SIM_TIMES: usize          = 50;   //50
///how many game sims can be running at once
const POOL_SIZE: usize          = 10;   //10
///generation size
const BATCH_SIZE: usize         = 200;  //200           try to keep div by 5
///how many generations to run
const GENERATIONS: usize        = 0;    //200           IF 0 THEN INFINITE
///max level before timeout
const MAX_LEVEL: usize          = 30;   //20
///how often to update screen
const DISPLAY_INTERVAL: usize   = 13;

const U_RANGE: (usize, usize)   = (0, 5);       //max 4
const F_RANGE: (f32, f32)       = (0.0, 1.0);
const U_NUDGE: usize            = 1;
const F_NUDGE: f32              = 0.1;

const BREEDER_PERCENT: f32      = 0.20; //should all add up to 100% try to keep div by 5
const PERCENT_CROSS: f32        = 0.70;
const PERCENT_INSERT: f32       = 0.00;
const PERCENT_RAND: f32         = 0.10;

const INSERT_CHANCE: f32        = 0.10; //% of chromosomes in insert pool will be inserted
const NUDGE_CHANCE: f32         = 0.10; //% of chromosomes will get nudged
const MUTATION_CHANCE: f32      = 0.05; //% of chromosomes will mutate


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

fn display_gen_info(total_start: Instant, past_elapsed: u64, start: Instant, gen: usize, results: &Vec<GameResult>) {
    let elapsed = (Instant::now()-start).as_secs();
    println!("GENERATION {} COMPLETED IN {}       |    {:0.2}g/s    |    {:>3}    |    score variation: {}",
        gen+1,
        format_time(elapsed, "hms"),
        (SIM_TIMES*BATCH_SIZE) as f32/elapsed as f32,
        results.iter().map(|r| r.placed).sum::<usize>()/results.len(),
        results[0].score-results[BATCH_SIZE-1].score
    );
    let session_elapsed = (Instant::now()-total_start).as_secs();
    let total_elapsed = (Instant::now()-total_start).as_secs()+past_elapsed;
    let eta = ((GENERATIONS as u64 * total_elapsed)
                .checked_div(gen as u64)
                .unwrap_or(0))
                .checked_sub(total_elapsed)
                .unwrap_or(0);
    print!("SESSION: {}    ",format_time(session_elapsed, "dhms"));
    if session_elapsed!=total_elapsed {
        print!("|    TOTAL: {}    ", format_time(total_elapsed, "dhms"));
    }
    if GENERATIONS != 0 {
        print!("|    TOTAL ETA: {}",format_time(eta, "dhms"));
    }
    println!();
    let disp_num = {if BATCH_SIZE >= 10 {10} else {BATCH_SIZE}};
    println!("Ao{}   {:>7} |   {:>2}",
        disp_num,
        results[0..disp_num].iter().map(|r| r.score).sum::<usize>()/disp_num,
        results[0..disp_num].iter().map(|r| r.level).sum::<usize>()/disp_num,
    );
    GameResult::print_header();
    let disp_num = {if BATCH_SIZE >= 5 {5} else {BATCH_SIZE}};
    for i in 0..disp_num {
        println!("  {:>2} |{}", i+1, results[i]);
    }
    println!("\n");
}



















macro_rules! u {
    ($x:expr) => {
        {if let Self::U(i) = $x {i} else {panic!("Invalid Param")}}
    };
}

macro_rules! f {
    ($x:expr) => {
        {if let Self::F(i) = $x {i} else {panic!("Invalid Param")}}
    };
}

macro_rules! params_parse {
    ($x:expr) => {
        $x.next().ok_or("Failed to parse")?.replace(" ","").parse()?
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
            min_lines_to_clear:             u!(params[0]),
            lines_cleared_importance:       f!(params[1]),
            points_scored_importance:       f!(params[2]),
            piece_depth_importance:         f!(params[3]),
            max_height_importance:          f!(params[4]),
            avg_height_importance:          f!(params[5]),
            height_variation_importance:    f!(params[6]),
            current_holes_importance:       f!(params[7]),
            max_pillar_height:              u!(params[8]),
            current_pillars_importance:     f!(params[9]),
        }
    }

    fn parse(mut params: Split<char>) -> DynResult<ai::AiParameters> {
        Ok(
            ai::AiParameters {
                min_lines_to_clear:             params_parse!(params),
                lines_cleared_importance:       params_parse!(params),
                points_scored_importance:       params_parse!(params),
                piece_depth_importance:         params_parse!(params),
                max_height_importance:          params_parse!(params),
                avg_height_importance:          params_parse!(params),
                height_variation_importance:    params_parse!(params),
                current_holes_importance:       params_parse!(params),
                max_pillar_height:              params_parse!(params),
                current_pillars_importance:     params_parse!(params),
            }
        )
    }
}


fn random_generation() -> Vec<ai::AiParameters> {
    let mut rng = rand::thread_rng();
    (0..BATCH_SIZE).map(|_| {
        ai::AiParameters {
            min_lines_to_clear:             rng.gen_range(U_RANGE.0, U_RANGE.1),
            lines_cleared_importance:       rng.gen_range(F_RANGE.0, F_RANGE.1),
            points_scored_importance:       rng.gen_range(F_RANGE.0, F_RANGE.1),
            piece_depth_importance:         rng.gen_range(F_RANGE.0, F_RANGE.1),
            max_height_importance:          rng.gen_range(F_RANGE.0, F_RANGE.1),
            avg_height_importance:          rng.gen_range(F_RANGE.0, F_RANGE.1),
            height_variation_importance:    rng.gen_range(F_RANGE.0, F_RANGE.1),
            current_holes_importance:       rng.gen_range(F_RANGE.0, F_RANGE.1),
            max_pillar_height:              rng.gen_range(U_RANGE.0, U_RANGE.1),
            current_pillars_importance:     rng.gen_range(F_RANGE.0, F_RANGE.1),
        }
    }).collect::<Vec<ai::AiParameters>>()
}



//attempts to get the progress of last species trained
fn get_progress() -> DynResult<Option<(usize, u64, Vec<GameResult>)>>{
    match File::open("species.log") {
        Ok(file) => {
            let mut prog = BufReader::new(file).lines().map(|l| Ok(l?)).collect::<DynResult<Vec<String>>>()?.into_iter();
            let header = prog.next().ok_or("Failed to parse")?;
            let mut header = header.split("|");
            let gen = params_parse!(header);
            let elapsed = params_parse!(header);
            let generation = prog.map(|line| {
                let mut fields = line.split('|');
                Ok(GameResult {
                    score: params_parse!(fields),
                    level: params_parse!(fields),
                    placed: params_parse!(fields),
                    parameters: Some(Params::parse(fields.next().ok_or("Failed to parse")?.split(':'))?),
                })
            }).collect::<DynResult<Vec<GameResult>>>()?;
            Ok(Some((gen, elapsed, generation)))
        },
        Err(e) if e.kind() == NotFound => {Ok(None)},
        Err(e) => dynerr!(e),
    }
}


//gets the best results, or empty vec if best.log doesnt exist
fn get_best_results() -> DynResult<Vec<(usize, GameResult)>> {
    match File::open("best.log") {
        Ok(file) => {
            let prog = BufReader::new(file).lines().map(|l| Ok(l?)).collect::<DynResult<Vec<String>>>()?.into_iter();
            let best_results = prog.map(|line| {
                let mut fields = line.split('|');
                let gen = params_parse!(fields); 
                Ok((gen, GameResult {
                    score: params_parse!(fields),
                    level: params_parse!(fields),
                    placed: params_parse!(fields),
                    parameters: Some(Params::parse(fields.next().ok_or("Failed to parse")?.split(':'))?),
                }))
            }).collect::<DynResult<Vec<(usize, GameResult)>>>()?;
            Ok(best_results)
        }
        Err(e) if e.kind() == NotFound => {Ok(Vec::new())},
        Err(e) => dynerr!(e),
    }
}

//needs to take entire list and do the whole "more likely to breed better ones" thing
//because right now im getting convergence way too early
//it'll allow me to keep a larger pool of diversity by not trimming 80% every gen
///takes breeders and breeds next generation
fn breed_next_gen(breeders: &[GameResult]) -> Vec<ai::AiParameters> {
    assert_eq!(breeders.len(), (BATCH_SIZE as f32*BREEDER_PERCENT) as usize);
    let mut rng = rand::thread_rng();
    let mut kids = Vec::with_capacity(BATCH_SIZE);
    let params = breeders.iter().map(|b|
        Params::get(&b.parameters.as_ref().unwrap())
    ).collect::<Vec<[Params;10]>>();
    //closure to get random pair
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
    //crossover
    for _ in 0..(BATCH_SIZE as f32*PERCENT_CROSS) as usize/2 {
        let (male, fema) = get_couple(rng);
        let mut kid1 = male.clone();
        let mut kid2 = fema.clone();
        let divide = rng.gen_range(0, kid1.len());
        for x in 0..divide {mem::swap(&mut kid1[x], &mut kid2[x])}
        kids.push(kid1);
        kids.push(kid2);
    }
    //insert
    for _ in 0..(BATCH_SIZE as f32*PERCENT_INSERT) as usize {
        let (male, fema) = get_couple(rng);
        let mut kid = [Params::U(0);10];
        for x in 0..kid.len() {
            if rng.gen_range(F_RANGE.0, F_RANGE.1) <= INSERT_CHANCE {kid[x] = fema[x]}
            else {kid[x] = male[x]}
        }
        kids.push(kid);
    }
    //generate new randoms
    for _ in 0..(BATCH_SIZE as f32*PERCENT_RAND) as usize {
        let kid = [
            Params::U(rng.gen_range(U_RANGE.0, U_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
            Params::U(rng.gen_range(U_RANGE.0, U_RANGE.1)),
            Params::F(rng.gen_range(F_RANGE.0, F_RANGE.1)),
        ];
        kids.push(kid);
    }

    //nudge
    for nudge in &mut kids {
        for gene in nudge {
            if rng.gen_range(F_RANGE.0, F_RANGE.1) <= NUDGE_CHANCE {
                match gene {
                    Params::U(u) => {
                        match rng.gen_range(0,2) {
                            0 => *u = u.checked_sub(U_NUDGE).unwrap_or(U_RANGE.0),
                            _ => *u = if *u+U_NUDGE > U_RANGE.1 {U_RANGE.1} else {*u+U_NUDGE},
                        }
                    },
                    Params::F(f) => {
                        match rng.gen_range(0,2) {
                            0 => *f = if *f-F_NUDGE < 0.0 {F_RANGE.0} else {*f+F_NUDGE},
                            _ => *f = if *f+F_NUDGE > F_RANGE.1 {F_RANGE.1} else {*f+F_NUDGE},
                        }
                    },
                }
            }
        }
    }

    //mutate
    for cronenberg in &mut kids {
        for gene in cronenberg {
            if rng.gen_range(F_RANGE.0, F_RANGE.1) <= MUTATION_CHANCE {
                match gene {
                    Params::U(u) => *u = rng.gen_range(U_RANGE.0, U_RANGE.1),
                    Params::F(f) => *f = rng.gen_range(F_RANGE.0, F_RANGE.1),
                }
            }
        }
    }
    kids.extend(params);
    assert_eq!(kids.len(), BATCH_SIZE);
    kids.iter().map(|k| Params::construct(*k)).collect()
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

///saves current stats
fn log_stats(best_results: &Vec<(usize, GameResult)>, breeders: &[GameResult], gen: usize, total_start: Instant, past_elapsed: u64) {
    clean!("best.log");
    for res in best_results {log!(format!("{} |{}",res.0, res.1), "best.log");}
    clean!("species.log");
    log!(format!("{} | {}", gen, (Instant::now()-total_start).as_secs()+past_elapsed), "species.log");
    for res in breeders {log!(res, "species.log");}
}


///does the actual training
pub fn train() -> DynResult<()> {
    let (mut gen, past_elapsed, mut generation) = match check!(get_progress()) {
        Some((gen, elapsed, results)) => {
            println!("RESUMING SPECIES FROM GENERATION {}", gen);
            (gen, elapsed, breed_next_gen(&results))            
        },
        None => {
            println!("STARTING NEW SPECIES");
            (0, 0, random_generation())
        },
    };
    let mut best_results = check!(get_best_results());
    let total_start = Instant::now();
    loop {
        gen+=1;
        println!("STARTING GENERATION {}", gen);
        let start = Instant::now();
        let results = check!(do_generation(generation));
        display_gen_info(total_start, past_elapsed, start, gen-1, &results);
        //update and log best results
        best_results.extend(
            results[0..{if BATCH_SIZE >= 10 {10} else {BATCH_SIZE}}].iter().map(|r| 
                (gen, r.clone())
            ).collect::<Vec<(usize, GameResult)>>()
        );
        //update and log stats
        best_results.sort_by(|a, b| b.1.score.cmp(&a.1.score));
        if best_results.len() > 10 {best_results.drain(10..);}
        let breeders = &results[0..(BATCH_SIZE as f32*BREEDER_PERCENT) as usize];
        log_stats(&best_results, breeders, gen, total_start, past_elapsed);
        //breed
        generation = breed_next_gen(breeders);
        //generation logic
        if gen >= GENERATIONS && GENERATIONS!=0 {break}
    }
    let total_elapsed = Instant::now()-total_start;
    println!("{} generations completed in {}", GENERATIONS, format_time(total_elapsed.as_secs(), "dhms"));
    println!("Average of 1 generation every {}", format_time((total_elapsed/GENERATIONS as u32).as_secs(),"hms"));

    println!("BEST RESULTS");
    GameResult::print_header();
    let disp_num = {if GENERATIONS >= 10 {10} else {GENERATIONS}};
    for i in 0..disp_num {println!("  {:>2} | {} |{}", i+1, best_results[i].0, best_results[i].1)}
    Ok(())
}

//cargo run --release -- --train


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



//TO OPTIMIZE
//benchmark AI functions
//change pieces to vec<&bool> then shuffle references to rotate. avoids the clone required during rotation

//improve breeding function
//add minor mutations (add/sub 0.5 or something from current value to help nudge it out of pockets)



//had it catching up from 10 to 20. this is what it looked like before i set it on 30 and went to bed
// 53
//   954805 |   13  |    362 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.11643
//   861060 |   13  |    361 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.06300
//   837389 |   12  |    344 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.33975
//   813738 |   12  |    339 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.00861 : 0.42276 : 0 : 0.06300
//   809012 |   12  |    335 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.64071 : 0 : 0.36368
//   804263 |   12  |    333 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.33975
//   802046 |   15  |    407 | 4 : 0.69280 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.06300
//   798369 |   12  |    340 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.06300
//   778202 |   11  |    314 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.64071 : 0 : 0.14467
//   757560 |   12  |    343 | 4 : 0.80500 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.14467
//   739954 |   11  |    313 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.06300
//   738032 |   11  |    323 | 4 : 0.83073 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.43493 : 0 : 0.14467
//   735720 |   10  |    291 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 1 : 0.06300
//   724220 |   12  |    341 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 2 : 0.06300
//   714934 |   11  |    316 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.43493 : 0 : 0.14467
//   712811 |   10  |    293 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.62516 : 0 : 0.14467
//   706636 |    9  |    270 | 3 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.06300
//   697394 |   10  |    295 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.33975
//   693132 |   11  |    317 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 2 : 0.06300
//   691278 |   10  |    294 | 4 : 0.87685 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.14467
//   689514 |    9  |    277 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.71503 : 0 : 0.14467
//   683462 |   11  |    309 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.10977
//   676975 |   11  |    307 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.33975
//   675577 |   10  |    284 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.57338 : 0 : 0.40332
//   672015 |   10  |    292 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 1 : 0.06300
//   650116 |   10  |    282 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.43493 : 0 : 0.14467
//   649753 |   14  |    385 | 4 : 0.79000 : 0.00100 : 0.21726 : 0.23985 : 0.09200 : 0.01900 : 0.56915 : 0 : 0.33975
//   639344 |   10  |    290 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.33975
//   635224 |   10  |    294 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 2 : 0.06300
//   630741 |    9  |    265 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.06300
//   609152 |    9  |    273 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.51535 : 2 : 0.14467
//   604470 |    9  |    263 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.33975
//   602932 |    9  |    274 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 3 : 0.33975
//   600642 |    9  |    251 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.57338 : 0 : 0.40332
//   596260 |    9  |    262 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 2 : 0.06300
//   593022 |    9  |    271 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.43493 : 0 : 0.14467
//   582988 |    9  |    252 | 4 : 0.79000 : 0.00100 : 0.33926 : 0.05000 : 0.09200 : 0.01900 : 0.59161 : 0 : 0.36368
//   582616 |    5  |    182 | 4 : 0.87685 : 0.00100 : 0.13340 : 0.05000 : 0.09200 : 0.01900 : 0.56915 : 0 : 0.11643
//   578648 |    7  |    214 | 4 : 0.79000 : 0.00100 : 0.16135 : 0.05000 : 0.09200 : 0.01900 : 0.42276 : 0 : 0.06300
//   576082 |    6  |    196 | 4 : 0.79000 : 0.00100 : 0.21726 : 0.05000 : 0.09200 : 0.01900 : 0.62823 : 0 : 0.11643