use super::*;

use std::thread;
use std::time::Duration;

///handle to the thread doing updates
pub struct DisplayThread {
    tx: mpsc::Sender<bool>,
    handle: thread::JoinHandle<()>,
}

impl DisplayThread {
    ///starts a display thread that prints progress statistics every x seconds
    pub fn start(prog: Arc<Mutex<usize>>) -> Self {
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
    pub fn stop(self) -> DynResult<()> {
        self.tx.send(true)?;
        self.handle.join().unwrap();
        Ok(())
    }
}




///displays the progress update after each generation has completed
pub fn display_gen_info(time_handle: &TimeTracker, gen: usize, results: &Vec<GameResult>) {
    let loop_time = time_handle.loop_elapsed();
    println!("GENERATION {} COMPLETED IN {}       |    {:0.2}g/s    |    {:>3}    |    score variation: {}",
        gen+1,
        format_time(loop_time, "hms"),
        (SIM_TIMES*BATCH_SIZE) as f32/loop_time as f32,
        results.iter().map(|r| r.placed).sum::<usize>()/results.len(),
        results[0].score-results[BATCH_SIZE-1].score
    );
    let training_time = time_handle.total_training();
    let eta = ((GENERATIONS as u64 * training_time)
                .checked_div(gen as u64)
                .unwrap_or(0))
                .checked_sub(training_time)
                .unwrap_or(0);
    let session_elapsed = time_handle.session_elapsed();
    print!("SESSION: {}    ",format_time(session_elapsed, "dhms"));
    if session_elapsed!=training_time {
        print!("|    TOTAL: {}    ", format_time(training_time, "dhms"));
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
        println!("  {:>2} | {}", i+1, results[i]);
    }
    println!("\n");
}



///formats time to supplied string format of values "dhms"
pub fn format_time(seconds: u64, formatting: &str) -> String {
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



pub struct TimeTracker {
    started: Instant,
    loop_start: Instant,
    previous_elapsed: u64,
}

impl TimeTracker {
    pub fn new(previous_elapsed: u64) -> Self {
        Self {
            started: Instant::now(),
            loop_start: Instant::now(),
            previous_elapsed,
        }
    }

    pub fn start_loop(&mut self) {
        self.loop_start = Instant::now();
    }

    fn loop_elapsed(&self) -> u64 {
        (Instant::now()-self.loop_start).as_secs()
    }

    fn session_elapsed(&self) -> u64 {
        (Instant::now()-self.started).as_secs()
    }

    pub fn total_training(&self) -> u64 {
        (Instant::now()-self.started).as_secs()+self.previous_elapsed
    }
}