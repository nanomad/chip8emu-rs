#![feature(try_from)]
extern crate minifb;
extern crate rand;

use std::env;

mod emulator;
mod instruction;
mod chip8;
mod video_engine;
mod debugger;
mod peripherals;

use emulator::Emulator;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    println!("RUST Chip8 Emulator");

    let rom_path = env::args().nth(1).expect("Please provide a path to a Chip8 ROM");
    let rom = load_rom(&rom_path);
    let mut emulator = Emulator::new(&rom);
    emulator.run();

}


fn load_rom(path: &str) -> Vec<u8> {
    let mut f: File = File::open(path).expect(&format!("Cannot open file {}", path));
    let mut buf: Vec<u8> = Vec::new();
    let read = f.read_to_end(&mut buf).expect(&format!("Cannot read from file {}", path));
    println!("Loaded {} bytes", read);
    buf
}
