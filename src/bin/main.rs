extern crate tetris;
use std::env::args;

fn main() {
    let train = match args().nth(1) {
        Some(arg) if arg == "--train" => true,
        _  => false,
    };
    tetris::run(train);
}