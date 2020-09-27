use super::*;

use dynerr::*;

use std::io::ErrorKind::NotFound;
use std::str::Split;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;
use std::fmt;

///parses T from string
macro_rules! params_parse {
    ($x:expr) => {$x.next().ok_or("Failed to parse")?.replace(" ","").parse()?};
}


///takes iterator of strings representing GameResults and attempts to parse it
fn parse_game_result(mut fields: Split<char> ) -> DynResult<GameResult> {
    Ok(GameResult {
        score: params_parse!(fields),
        level: params_parse!(fields),
        placed: params_parse!(fields),
        parameters: Some(ai::AiParameters::construct(
            fields.next().ok_or("Failed to parse params")?.split(':').map(|p|
                Ok(p.replace(" ","").parse()?)
            ).collect::<DynResult<Vec<f32>>>()?.as_slice().try_into()?
        ))
    })
}


//attempts to get the generation num, elapsed time, and last saved results of last species trained
pub fn get_progress() -> DynResult<Option<(usize, u64, Vec<GameResult>)>>{
    match File::open("species.log") {
        Ok(file) => {
            let mut prog = BufReader::new(file)
                .lines()
                .map(|l| Ok(l?))
                .collect::<DynResult<Vec<String>>>()?
                .into_iter();
            let pre_header = prog.next().ok_or("Failed to parse")?;
            let mut header = pre_header.split('|');
            let gen = params_parse!(header);
            let elapsed = params_parse!(header);
            let generation = prog.map(|line| {
                parse_game_result(line.split('|'))
            }).collect::<DynResult<Vec<GameResult>>>()?;
            Ok(Some((gen, elapsed, generation)))
        },
        Err(e) if e.kind() == NotFound => {Ok(None)},
        Err(e) => dynerr!(e),
    }
}



///saves current stats
pub fn log_stats(best_results: &Vec<BestResult>, breeders: &[GameResult], gen: usize, time_handle: &display::TimeTracker) {
    clean!("best.log");
    for res in best_results {log!(res, "best.log");}
    clean!("species.log");
    log!(format!("{} | {}", gen, time_handle.total_training()), "species.log");
    for res in breeders {log!(res, "species.log");}
}



#[derive(PartialEq)]
pub struct BestResult {
    pub gen: usize,
    pub result: GameResult,
}

impl BestResult {
    //gets the best results from last session, or empty vec if best.log doesnt exist
    pub fn get() -> DynResult<Vec<Self>> {
        match File::open("best.log") {
            Ok(file) => {
                let prog = BufReader::new(file)
                    .lines()
                    .map(|l| Ok(l?))
                    .collect::<DynResult<Vec<String>>>()?
                    .into_iter();
                let best_results = prog.map(|line| {
                    let mut fields = line.split('|');
                    Ok(Self {
                        gen: params_parse!(fields),
                        result: parse_game_result(fields)?,
                    })
                }).collect::<DynResult<Vec<Self>>>()?;
                Ok(best_results)
            }
            Err(e) if e.kind() == NotFound => {Ok(Vec::new())},
            Err(e) => dynerr!(e),
        }
    }

    pub fn update(best: &mut Vec<Self>, results: &Vec<GameResult>, gen: usize) {
        best.extend(
            results[0..{if BATCH_SIZE >= 10 {10} else {BATCH_SIZE}}].iter().map(|r| 
                progress::BestResult{gen, result: *r}
            ).collect::<Vec<progress::BestResult>>()
        );
        best.sort_by(|a, b| b.result.score.cmp(&a.result.score));
        best.dedup_by(|a, b| a.result.parameters.eq(&b.result.parameters));
        if best.len() > 10 {best.drain(10..);}
    }
}

impl fmt::Display for BestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{:>4} | {}",
            self.gen,
            self.result
        )
    }
}