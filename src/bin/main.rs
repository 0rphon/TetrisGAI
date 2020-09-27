extern crate tetris;
use std::env::args;

fn main() {
    let (train, auto_loop) = match args().nth(1) {
        Some(arg) if arg == "--train"     => (true, false),
        Some(arg) if arg == "--auto-loop" => (false, true),
        _  => (false, false),
    };
    tetris::run(train, auto_loop);
}