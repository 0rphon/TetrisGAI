#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate tetris;
use std::env::args;

const HELP_LOG: &str = "TetrisGAI: Why go through the work of playing tetris when you could just automate it?

--auto-loop:        Allow the AI to restart games on its own.

--train:            Train your own AI. See the constants in train.rs to change how the AI is trained. 
                    Can only be used in debug builds and cant be used with other commands.
                    Still needs work.

--use_best:         Use the top result from training. Stored in the top line of best.log.

--help:             Show this command and exit.";

fn main() {
    let arguments = args().skip(1);
    let mut settings = (false, false, false);
    for arg in arguments {
        match arg {
            arg if arg == "--train"     => {
                if cfg!(debug_assertions) {settings.0 = true} 
                else {panic!("--train can only be used in debug builds! Try --help.")}
            },
            arg if arg == "--auto-loop" => settings.1 = true,
            arg if arg == "--use_best"  => settings.2 = true,
            arg if arg == "--help"      => {println!("{}",HELP_LOG); return},
            arg => panic!("Unknown argument \"{}\". Try --help",arg)
        };
    }
    if settings.0 && (settings.1||settings.2) {panic!("--train is mutually exclusive! Try --help.")}
    tetris::run(settings.0, settings.1, settings.2);
}