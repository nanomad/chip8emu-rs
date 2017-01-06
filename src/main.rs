#![feature(try_from)]
extern crate minifb;

use minifb::{Key, WindowOptions, Window, KeyRepeat};
use std::sync::mpsc::channel;
use std::thread;
use std::io::prelude::*;
use std::io;
use std::usize;

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

    let mut window = Window::new("Test - ESC to exit", 64, 32, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let mut video_engine = VideoEngine::new(window);
    let mut chip8 = Chip8::new();
    chip8.load_rom("data\\BLITZ");
    let mut debugger = Debugger::new(chip8);
    while video_engine.is_running() {
        if !debugger.run(&mut video_engine) {
            break;
        }
    }
}
