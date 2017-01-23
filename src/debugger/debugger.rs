use std::io;
use std::io::prelude::*;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use std::convert::TryFrom;
use debugger::command::Command;
use chip8::Chip8;
use instruction::Instruction;
use video_engine;
use peripherals::Peripherals;
use std::sync::mpsc::Receiver;

pub struct Debugger {
    breakpoints: HashSet<usize>,
    cursor: usize,
    last_command: Option<Command>,
    exit: bool,
}

impl Debugger {
    pub fn new() -> Self {
        Debugger {
            breakpoints: HashSet::new(),
            cursor: 0,
            last_command: None,
            exit: false,
        }
    }

    fn step(&mut self, chip8: &mut Chip8, peripherals: &mut Peripherals) {
        self.disam_instr(chip8, chip8.pc());
        chip8.step(peripherals);
        thread::sleep(Duration::from_millis(10));
    }

    pub fn manage_cli(&mut self,
                      stdin_receiver: &mut Receiver<String>,
                      chip8: &mut Chip8,
                      peripherals: &mut Peripherals)
                      -> bool {
        let mut again = true;
        if let Ok(command_string) = stdin_receiver.try_recv() {
            match Command::try_from(command_string) {
                Ok(cmd) => {
                    println!("Executing {:?}", cmd);
                    self.cursor = chip8.pc();
                    again = self.execute_command(cmd, chip8, peripherals);
                }
                Err(message) => {
                    println!("{}", message);
                }
            }
            if again {
                print!("[0x{:2x}]> ", chip8.pc());
                io::stdout().flush().expect("Could not flush stdout");
            }
        }
        again
    }

    fn execute_command(&mut self,
                       cmd: Command,
                       chip8: &mut Chip8,
                       peripherals: &mut Peripherals)
                       -> bool {
        match cmd {
            Command::Repeat => {}
            _ => self.last_command = Some(cmd),
        };
        match cmd {
            Command::Break { loc } => {
                self.add_breakpoint(loc);
                true
            }
            Command::Step => {
                self.step(chip8, peripherals);
                true
            }
            Command::Run => false,
            Command::Quit => {
                self.exit = true;
                false
            }
            Command::Dump { count } => {
                for i in 0..count {
                    let pos = self.cursor + i;
                    println!("[0x{:03x}] 0x{:x}", pos, chip8.mem()[pos])
                }
                true
            }
            Command::VideoRamDump => {
                let vram = peripherals.video_engine.vram();
                for (i, vram_i) in vram.iter().enumerate() {
                    if i > 0 && i % 64 == 0 {
                        println!();
                    }
                    match *vram_i {
                        video_engine::BACK_COLOR => print!("0"),
                        video_engine::FRONT_COLOR => print!("1"),
                        other => panic!("Color {:x} not supported", other),
                    }
                }
                println!();
                true
            }
            Command::RegDump => {
                let regs = chip8.reg_v();
                for (i, reg) in regs.iter().enumerate() {
                    if i > 0 && i % 4 == 0 {
                        println!();
                    }
                    print!("[{:02}]0x{:03x} ", i, reg)
                }
                println!();
                println!("{}", String::from_utf8(vec![b'-'; 39]).unwrap());
                print!("[i ]0x{:04x} ", chip8.reg_i());
                print!("[d  ]0x{:03x} ", chip8.reg_delay_timer());
                println!("[s ]0x{:03x}", chip8.reg_sound_timer());
                true
            }
            Command::StackDump => {
                let stack = chip8.stack();
                let sp = chip8.sp();
                for (i, stack_i) in stack.iter().enumerate() {
                    let x = if i == sp { "*" } else { " " };
                    println!("[{:}{:02}]0x{:x} ", x, i, stack_i)
                }
                true
            }
            Command::Disasm { count } => {
                for i in 0..count {
                    let mem_pos = self.cursor + 2 * i;
                    self.disam_instr(chip8, mem_pos)
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
                    Some(last_command) => self.execute_command(last_command, chip8, peripherals),
                }
            }
        }
    }

    fn disam_instr(&self, chip8: &Chip8, mem_pos: usize) {
        let hi_nibble = chip8.mem()[mem_pos] as u16;
        let lo_nibble = chip8.mem()[mem_pos + 1] as u16;
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

    pub fn must_break(&self, chip8: &Chip8) -> bool {
        let loc = chip8.pc();
        (!self.breakpoints.is_empty() && self.breakpoints.contains(&loc))
    }

    pub fn is_exit(&self) -> bool {
        self.exit
    }
}
