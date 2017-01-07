use std::collections::HashSet;
use std::io::prelude::*;
use std::thread;
use std::io;
use std::convert::TryFrom;
use super::command::Command;
use super::super::chip8::Chip8;
use super::super::instruction::Instruction;
use super::super::video_engine;
use super::super::video_engine::VideoEngine;

#[derive(Debug)]
pub enum ExecutionMode {
    Exit,
    Step,
    RunUntilBreakpoint,
}

pub struct Debugger {
    breakpoints: HashSet<usize>,
    execution_mode: ExecutionMode,
    chip8: Chip8,
    cursor: usize,
    on_breakpoint: bool,
}

impl Debugger {
    pub fn new(chip8: Chip8) -> Self {
        Debugger {
            breakpoints: HashSet::new(),
            execution_mode: ExecutionMode::Step,
            chip8: chip8,
            cursor: 0,
            on_breakpoint: false,
        }
    }

    pub fn run(&mut self, video_engine: &mut VideoEngine) -> bool {
        self.cursor = self.chip8.pc();
        while {
            let cmd = self.read_stdin();
            self.execute_command(cmd, video_engine)
        } {
            // Nothing, just read the next command until we must execute
        }
        match self.execution_mode {
            ExecutionMode::Exit => false,
            ExecutionMode::Step => {
                self.step(video_engine);
                true
            }
            ExecutionMode::RunUntilBreakpoint => {
                self.run_until_breakpoint(video_engine);
                true
            }
        }
    }

    fn run_until_breakpoint(&mut self, video_engine: &mut VideoEngine) {
        while !self.must_break(self.chip8.pc()) {
            self.step(video_engine);
        }
        self.on_breakpoint = true;
    }

    fn step(&mut self, video_engine: &mut VideoEngine) {
        self.disam_instr(self.chip8.pc());
        self.chip8.step(video_engine);
        self.on_breakpoint = false;
        thread::sleep_ms(100);
    }

    fn execute_command(&mut self, cmd: Command, video_engine: &VideoEngine) -> bool {
        match cmd {
            Command::Break { loc } => {
                self.add_breakpoint(loc);
                true
            }
            Command::Step => {
                self.execution_mode = ExecutionMode::Step;
                false
            }
            Command::Run => {
                self.execution_mode = ExecutionMode::RunUntilBreakpoint;
                false
            }
            Command::Quit => {
                self.execution_mode = ExecutionMode::Exit;
                false
            }
            Command::Dump => {
                println!("[0x{:03x}] 0x{:x}",
                         self.cursor,
                         self.chip8.mem()[self.cursor]);
                true
            }
            Command::VideoRamDump => {
                let vram = video_engine.vram();
                for i in 0..vram.len() {
                    if i > 0 && i % 64 == 0 {
                        println!();
                    }
                    match vram[i] {
                        video_engine::BACK_COLOR => print!("0"),
                        video_engine::FRONT_COLOR => print!("1"),
                        other => panic!("Color {:x} not supported", other),
                    }
                }
                println!();
                true
            }
            Command::Disasm { count } => {
                for i in 0..count {
                    let mem_pos = self.cursor + 2 * i;
                    self.disam_instr(mem_pos)
                }
                true
            }
            Command::Goto { loc } => {
                self.cursor = loc;
                true
            }
            _ => {
                println!("Command not yet implemented {:?}", cmd);
                true
            }
        }
    }

    fn disam_instr(&self, mem_pos: usize) {
        let hi_nibble = self.chip8.mem()[mem_pos] as u16;
        let lo_nibble = self.chip8.mem()[mem_pos + 1] as u16;
        let opcode = (hi_nibble << 8) | lo_nibble;
        match Instruction::try_from(opcode) {
            Ok(instruction) => println!("0x{0:03x} {1:?}", mem_pos, instruction),
            Err(_) => println!("0x{0:03x} dw 0x{1:x}", mem_pos, opcode),
        }
    }

    fn add_breakpoint(&mut self, loc: usize) {
        println!("Breakpoint installed at 0x{:03x}", loc);
        self.breakpoints.insert(loc);
    }

    fn must_break(&self, loc: usize) -> bool {
        (!self.on_breakpoint && !self.breakpoints.is_empty() && self.breakpoints.contains(&loc))
    }

    fn read_stdin(&self) -> Command {
        print!("(0x{:03x})> ", self.chip8.pc());
        io::stdout().flush().ok().expect("Could not flush stdout");
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut buffer).ok().expect("Cannot read from stdin");
        Command::from(buffer)
    }
}
