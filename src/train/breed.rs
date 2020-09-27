use super::*;

use std::mem;
use std::cmp::Ordering::Equal;
use rand::rngs::ThreadRng;
use rand::Rng;

///generates a random set of AI parameters
fn random_param() -> ai::AiParameters {
    let mut rng = rand::thread_rng();
    ai::AiParameters {
        min_lines_to_clear:             rng.gen_range(U_RANGE.0, U_RANGE.1+1) as f32,
        lines_cleared_importance:       rng.gen_range(F_RANGE.0, F_RANGE.1),
        points_scored_importance:       rng.gen_range(F_RANGE.0, F_RANGE.1),
        piece_depth_importance:         rng.gen_range(F_RANGE.0, F_RANGE.1),
        max_height_importance:          rng.gen_range(F_RANGE.0, F_RANGE.1),
        avg_height_importance:          rng.gen_range(F_RANGE.0, F_RANGE.1),
        height_variation_importance:    rng.gen_range(F_RANGE.0, F_RANGE.1),
        current_holes_importance:       rng.gen_range(F_RANGE.0, F_RANGE.1),
        max_pillar_height:              rng.gen_range(U_RANGE.0, U_RANGE.1+1) as f32,
        current_pillars_importance:     rng.gen_range(F_RANGE.0, F_RANGE.1),
    }
}



///generates BATCH_SIZE random parameters to seed a brand new species
pub fn new_species() -> Vec<ai::AiParameters> {
    (0..BATCH_SIZE).map(|_| {random_param()}).collect::<Vec<ai::AiParameters>>()
}


//gets two different random sets of ai parameters from vec
fn get_couple(params: &Vec<[f32;10]>, mut rng: ThreadRng) -> ([f32;10], [f32;10]){
    let couple = {
        let (mut x, mut y) = (rng.gen_range(0,params.len()), rng.gen_range(0,params.len()));
        while x==y {
            x = rng.gen_range(0,params.len());
            y = rng.gen_range(0,params.len());
        }
        (x,y)
    };
    (params[couple.0], params[couple.1])
}



//performs evolutionary gene crossover
fn crossover_genes(params: &Vec<[f32;10]>, mut rng: ThreadRng) -> Vec<[f32;10]> {
    let mut kids = Vec::new();
    for _ in 0..(BATCH_SIZE as f32*PERCENT_CROSS) as usize/2 {
        let (male, fema) = get_couple(params, rng);
        let mut kid1 = male;
        let mut kid2 = fema;
        let divide = rng.gen_range(0, kid1.len());
        for x in 0..divide {mem::swap(&mut kid1[x], &mut kid2[x])}
        kids.push(kid1);
        kids.push(kid2);
    }
    kids
}



//swaps random genes
fn insert_genes(params: &Vec<[f32;10]>, mut rng: ThreadRng) -> Vec<[f32;10]> {
    let mut kids = Vec::new();
    for _ in 0..(BATCH_SIZE as f32*PERCENT_INSERT) as usize {
        let (male, fema) = get_couple(params, rng);
        let mut kid = [0.0;10];
        for x in 0..kid.len() {
            if rng.gen_range(0.0, 1.0) <= INSERT_CHANCE {kid[x] = fema[x]}
            else {kid[x] = male[x]}
        }
        kids.push(kid);
    }
    kids
}



//mutates random genes by adding and subtracting small amounts
fn nudge_genes(kids: &mut Vec<[f32;10]>, mut rng: ThreadRng) {
    for nudge in kids {
        for (i, gene) in nudge.iter_mut().enumerate() {
            if rng.gen_range(0.0, 1.0) <= NUDGE_CHANCE {
                match i {
                    0|8 => {
                        match rng.gen_range(0,2) {
                            0 => *gene = if *gene-U_NUDGE as f32 <= U_RANGE.0 as f32 {U_RANGE.0 as f32} else {*gene-U_NUDGE as f32},
                            _ => *gene = if *gene+U_NUDGE as f32 >= U_RANGE.1 as f32 {U_RANGE.1 as f32} else {*gene+U_NUDGE as f32},
                        }
                    }
                    _ => {
                        let f_nudge = rng.gen_range(F_NUDGE_RANGE.0, F_NUDGE_RANGE.1);
                        match rng.gen_range(0,2) {
                            0 => *gene = if *gene-f_nudge <= F_RANGE.0 {F_RANGE.0} else {*gene-f_nudge},
                            _ => *gene = if *gene+f_nudge >= F_RANGE.1 {F_RANGE.1} else {*gene+f_nudge},
                        }
                    }
                }
            }
        }
    }
}



//mutates random genes to random number in range
fn mutate_genes(kids: &mut Vec<[f32;10]>, mut rng: ThreadRng) {
    for cronenberg in kids {
        for (i, gene) in cronenberg.iter_mut().enumerate() {
            if rng.gen_range(0.0, 1.0) <= MUTATION_CHANCE {
                match i {
                    0|8 => *gene = rng.gen_range(U_RANGE.0, U_RANGE.1+1) as f32,
                    _   => *gene = rng.gen_range(F_RANGE.0, F_RANGE.1)
                }
            }
        }
    }
}


//needs to take entire list and do the whole "more likely to breed better ones" thing
//because right now im getting convergence way too early
//it'll allow me to keep a larger pool of diversity by not trimming 80% every gen
///takes breeders and breeds next generation
pub fn breed_next_gen(breeders: &[GameResult]) -> Vec<ai::AiParameters> {
    let rng = rand::thread_rng();
    let mut kids = Vec::with_capacity(BATCH_SIZE);
    let params = breeders.iter().map(|b|b.parameters.unwrap().deconstruct()).collect::<Vec<[f32;10]>>();
    //crossover
    kids.extend(crossover_genes(&params, rng));
    //insert
    kids.extend(insert_genes(&params, rng));
    //nudge
    nudge_genes(&mut kids, rng);
    //mutate
    mutate_genes(&mut kids, rng);
    //add on the breeders from last gen
    kids.extend(params);
    //convert
    let mut next_batch = kids.iter().map(|k| ai::AiParameters::construct(*k)).collect::<Vec<ai::AiParameters>>();
    //remove duplicates
    next_batch.sort_by(|a,b| a.partial_cmp(&b).unwrap_or(Equal));
    next_batch.dedup();
    //fill empty space
    for _ in 0..(BATCH_SIZE-next_batch.len()) as usize {next_batch.push(random_param())}
    assert_eq!(next_batch.len(), BATCH_SIZE);
    next_batch
}