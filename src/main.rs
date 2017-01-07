#![feature(try_from)]
extern crate minifb;

use minifb::{Key, WindowOptions, Scale, Window, KeyRepeat};
use std::sync::mpsc::channel;
use std::thread;
use std::io::prelude::*;
use std::io;
use std::usize;
use std::env;

mod instruction;
mod chip8;
mod video_engine;
mod debugger;
use std::time::Duration;

use chip8::Chip8;
use video_engine::VideoEngine;
use debugger::debugger::Debugger;
use debugger::command::Command;


fn main() {
    println!("RUST Chip8 Emulator");

    let rom_path = env::args().nth(1).expect("Please provide a path to a Chip8 ROM");

    let window_options = WindowOptions {
        borderless: false,
        title: true,
        resize: false,
        scale: Scale::X16,
    };

    let mut window = Window::new("RUST Chip8 Emulator - ESC to exit", 64, 32, window_options)
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
