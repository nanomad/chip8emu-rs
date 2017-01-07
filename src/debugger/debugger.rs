use std::collections::HashSet;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
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
    last_command: Option<Command>,
}

impl Debugger {
    pub fn new(chip8: Chip8) -> Self {
        Debugger {
            breakpoints: HashSet::new(),
            execution_mode: ExecutionMode::Step,
            chip8: chip8,
            cursor: 0,
            last_command: None,
        }
    }

    pub fn run(&mut self, video_engine: &mut VideoEngine) -> bool {
        self.cursor = self.chip8.pc();
        match self.execution_mode {
            ExecutionMode::Exit => false,
            ExecutionMode::Step => {
                self.manage_cli(video_engine);
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
        while !self.must_break() {
            self.step(video_engine);
        }
        self.execution_mode = ExecutionMode::Step;
    }

    fn step(&mut self, video_engine: &mut VideoEngine) {
        self.disam_instr(self.chip8.pc());
        self.chip8.step(video_engine);
        thread::sleep(Duration::from_millis(10));
    }

    fn manage_cli(&mut self, video_engine: &mut VideoEngine) {
        while {
            let cmd = self.read_stdin();
            self.execute_command(cmd, video_engine)
        } {
            // Nothing, just read the next command until we must execute
        }
    }

    fn execute_command(&mut self, cmd: Command, video_engine: &VideoEngine) -> bool {
        match cmd {
            Command::Repeat => {}
            _ => self.last_command = Some(cmd.clone()),
        };
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
            Command::Dump { count } => {
                for i in 0..count {
                    let pos = self.cursor + i;
                    println!("[0x{:03x}] 0x{:x}", pos, self.chip8.mem()[pos])
                }
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
            Command::RegDump => {
                let regs = self.chip8.reg_v();
                for i in 0..regs.len() {
                    if i > 0 && i % 4 == 0 {
                        println!();
                    }
                    print!("[{:02}]0x{:03x} ", i, regs[i])
                }
                println!();
                println!("{}", String::from_utf8(vec![b'-'; 39]).unwrap());
                print!("[i ]0x{:04x} ", self.chip8.reg_i());
                print!("[d  ]0x{:03x} ", self.chip8.reg_delay_timer());
                println!("[s ]0x{:03x}", self.chip8.reg_sound_timer());
                true
            }
            Command::StackDump => {
                let stack = self.chip8.stack();
                let sp = self.chip8.sp();
                for i in 0..stack.len() {
                    let x = if i == sp { "*" } else { " " };
                    println!("[{:}{:02}]0x{:x} ", x, i, stack[i])
                }
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
            Command::Repeat => {
                match self.last_command {
                    None => true,
                    Some(last_command) => self.execute_command(last_command, &video_engine),
                }
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

    fn must_break(&self) -> bool {
        let loc = self.chip8.pc();
        (!self.breakpoints.is_empty() && self.breakpoints.contains(&loc))
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
