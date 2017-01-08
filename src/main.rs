#![feature(try_from)]
extern crate minifb;
extern crate rand;

use minifb::{WindowOptions, Scale, Window};
use std::env;

mod instruction;
mod chip8;
mod video_engine;
mod debugger;

use chip8::Chip8;
use video_engine::VideoEngine;
use debugger::debugger::Debugger;


fn main() {
    println!("RUST Chip8 Emulator");

    let rom_path = env::args().nth(1).expect("Please provide a path to a Chip8 ROM");

    let window_options = WindowOptions {
        borderless: false,
        title: true,
        resize: false,
        scale: Scale::X16,
    };

    let window = Window::new("RUST Chip8 Emulator - ESC to exit", 64, 32, window_options)
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let mut video_engine = VideoEngine::new(window);
    let mut chip8 = Chip8::new();
    chip8.load_rom(rom_path.as_str());
    let mut debugger = Debugger::new(chip8);
    while video_engine.is_running() {
        if !debugger.run(&mut video_engine) {
            break;
        }
    }
}
