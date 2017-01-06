#![feature(try_from)]
extern crate minifb;

use minifb::{Key, WindowOptions, Window, KeyRepeat};
use std::thread;
use std::io::prelude::*;
use std::io;
use std::usize;

mod instruction;
mod chip8;
mod video_engine;
mod debugger;

use chip8::Chip8;
use video_engine::VideoEngine;
use debugger::Debugger;


fn main() {
    println!("RUST Chip8 Emulator");


    let mut window = Window::new("Test - ESC to exit", 64, 32, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let mut video_engine = VideoEngine::new();
    let mut debugger = Debugger::new();
    let mut chip8 = Chip8::new();
    chip8.load_rom("C:\\Users\\nanomad\\IdeaProjects\\imapread-rs\\data\\BLITZ");
    debugger.add_breakpoint(0x225);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if debugger.is_paused() {
            match read_stdin() {
                Command::Break {loc} => {
                    debugger.add_breakpoint(loc);
                },
                Command::Run => {
                    debugger.reset()
                },
                Command::Step {count} => {
                    debugger.step(count)
                }
            }
        } else {
            chip8.step(&mut video_engine, &mut debugger);
            window.update_with_buffer(&video_engine.buffer());
            thread::sleep_ms(10);
        }
    }
}

#[derive(Debug)]
enum Command {
    Break { loc: usize },
    Step { count: usize },
    Run,
}

fn read_stdin() -> Command {
    print!("(paused)> ");
    io::stdout().flush().ok().expect("Could not flush stdout");
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut buffer).ok().expect("Cannot read from stdin");
    println!("Command: {}", buffer);
    parse_command(&buffer)
}

fn parse_command(text: &str) -> Command {
    let tokens: Vec<&str> = text.split(" ").collect();
    if !tokens.is_empty() {
        match tokens[0] {
            "break" | "b" => {
                Command::Break {
                    loc: usize::from_str_radix(tokens[1], 16)
                        .expect("Cannot parse command"),
                }
            }
            _ => Command::Run,
        }
    } else {
        Command::Run
    }
}
