extern crate tetris;
use std::env::args;

fn main() {
    let train = match args().nth(1) {
        Some(arg) if arg == "--train" => true,
        _  => false,
    };
    let dry = match args().nth(2) {
        Some(arg) if arg == "--dry" => true,
        _  => false,
    };
    tetris::run(train, dry);
}