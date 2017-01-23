use std::convert::TryFrom;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};
use std::thread::JoinHandle;

use instruction::Instruction;
use peripherals::Peripherals;

use rand::{thread_rng, Rng};

const FONT_BASE_ADDR: usize = 0x0;
const NUM_FONTS: usize = 16;
const FONT_SIZE: usize = 5;
const FONT_MAP: [u8; NUM_FONTS * FONT_SIZE] =
    [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0,
     0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
     0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0,
     0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
     0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0,
     0xF0, 0x80, 0xF0, 0x80, 0x80];

pub struct Chip8 {
    mem: Vec<u8>,
    reg_v: Vec<u8>,
    reg_i: u16,
    reg_delay_timer: Arc<AtomicUsize>,
    reg_sound_timer: u8,
    pc: usize,
    sp: usize,
    stack: Vec<usize>,
}

impl Chip8 {
    pub fn new(rom: &Vec<u8>) -> Chip8 {
        let mut chip8 = Chip8 {
            mem: vec![0; 0xFFF],
            reg_v: vec![0; 16],
            reg_i: 0,
            reg_delay_timer: Arc::new(AtomicUsize::new(0)),
            reg_sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: vec![0; 16],
        };
        chip8.load_fonts();
        chip8.load_rom(rom);
        chip8
    }

    fn load_fonts(&mut self) {
        for i in FONT_BASE_ADDR..FONT_MAP.len() {
            self.mem[i] = FONT_MAP[i - FONT_BASE_ADDR]
        }
    }

    fn load_rom(&mut self, rom: &Vec<u8>) {
        for (b, buf_i) in rom.iter().enumerate() {
            self.mem[0x200 + b] = *buf_i;
        }
    }

    pub fn step(&mut self, peripherals: &mut Peripherals) {
        let hi_nibble = self.mem[self.pc] as u16;
        let lo_nibble = self.mem[self.pc + 1] as u16;
        let opcode = (hi_nibble << 8) | lo_nibble;
        match Instruction::try_from(opcode) {
            Err(msg) => panic!("Error decoding instruction at 0x{0:03x}: {1}", self.pc, msg),
            Ok(instruction) => self.step_instruction(instruction, peripherals),
        }
    }

    fn memory_read(&self, pos: usize) -> u8 {
        self.mem[pos]
    }

    fn memory_write(&mut self, pos: usize, data: u8) {
        self.mem[pos] = data;
        // println!("[W][0x{:03x}] 0x{:03x}", pos, data);
    }

    fn step_instruction(&mut self, instruction: Instruction, peripherals: &mut Peripherals) {
        match instruction {
            Instruction::Cls => peripherals.video_engine.cls(),
            Instruction::Ret => {
                assert!(self.sp > 0);
                self.sp -= 1;
                let old_pc = self.stack[self.sp];
                println!("Resuming execution at PC 0x{:x}", old_pc);
                self.pc = old_pc; // Jump to the instruction immediately after
            }
            Instruction::Jmp { addr } => self.pc = addr - 2, // Correct for pc increment later
            Instruction::Jsr { addr } => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = addr - 2
            } // Correct for pc increment later
            Instruction::Mov { vr, k } => self.reg_v[vr] = k,
            Instruction::Movr { vr, vy } => self.reg_v[vr] = self.reg_v[vy],
            Instruction::And { vr, vy } => self.reg_v[vr] &= self.reg_v[vy],
            Instruction::Shr { vr } => {
                let current_val = self.reg_v[vr];
                self.reg_v[0xF] = current_val & 1;
                self.reg_v[vr] = current_val >> 1;
            }
            Instruction::Skner { vr, vy } => {
                if self.reg_v[vr] != self.reg_v[vy] {
                    self.pc += 2;
                }
            }
            Instruction::Skeq { vr, k } => {
                if self.reg_v[vr] == k {
                    self.pc += 2
                }
            }
            Instruction::Skne { vr, k } => {
                if self.reg_v[vr] != k {
                    self.pc += 2
                }
            }
            Instruction::Add { vr, k } => {
                let old_val = self.reg_v[vr];
                self.reg_v[vr] = old_val.wrapping_add(k);
            }
            Instruction::Addr { vr, vy } => {
                let old_r = self.reg_v[vr];
                let old_y = self.reg_v[vy];
                let (result, overflow) = old_r.overflowing_add(old_y);
                if overflow {
                    self.reg_v[0xF] = 1
                } else {
                    self.reg_v[0xF] = 0
                }
                self.reg_v[vr] = result;
            }
            Instruction::Mvi { k } => self.reg_i = k,
            Instruction::Rnd { vr, k } => {
                let mut buffer = [0u8; 1];
                thread_rng().fill_bytes(&mut buffer);
                self.reg_v[vr] = buffer[0] & k;
            }
            Instruction::Sprite { rx, ry, s } => {
                let x = self.reg_v[rx] as usize;
                let y = self.reg_v[ry] as usize;
                let height = s;
                self.reg_v[0xF] = 0;
                for yline in 0..height {
                    let mem_pos = self.reg_i as usize + yline as usize;
                    let pixel = self.memory_read(mem_pos);
                    for xline in 0..8 {
                        if pixel & (0x80 >> xline) != 0 {
                            let collision = peripherals.video_engine
                                .set_pixel_to_1(x + xline, y + yline);
                            if collision {
                                self.reg_v[0xF] = 1;
                            }
                        }

                    }
                }
            }
            Instruction::Skp { k } => {
                println!("Skipping if key {:x} is pressed", k);
                let key = peripherals.keypad.get_current_key_input();
                match key {
                    Some(x) if x == k => self.pc += 2,
                    _ => {}
                }
            }
            Instruction::Sknp { k } => {
                println!("Skipping if key {:x} is not pressed", k);
                let key = peripherals.keypad.get_current_key_input();
                match key {
                    Some(x) if x == k => {}
                    _ => self.pc += 2,
                }
            }
            Instruction::Key { vr } => {
                println!("Waiting for a key to be pressed");
                let key = peripherals.keypad.get_current_key_input();
                match key {
                    Some(x) => {
                        println!(" ... Got {:x}", x);
                        self.reg_v[vr] = x
                    }
                    _ => {
                        println!(" ... Waiting more");
                        self.pc -= 2; // Emulate a SLEEP
                    }
                }

            }
            Instruction::Adi { vr } => {
                self.reg_i += self.reg_v[vr] as u16;
            }
            Instruction::Font { vr } => {
                let character = self.reg_v[vr] as usize;
                self.reg_i = (FONT_BASE_ADDR + (character * FONT_SIZE)) as u16;
                println!("Font draw for character: {:x}", character);
            }
            Instruction::Bcd { vr } => {
                let value = self.reg_v[vr];
                println!("BCD DECODING OF {}", value);
                let i = self.reg_i as usize;
                self.mem[i] = value / 100;
                self.mem[i + 1] = (value / 10) % 10;
                self.mem[i + 2] = (value % 100) % 100;
            }
            Instruction::Str { vr } => {
                for idx in 0..(vr + 1) {
                    let target_pos = self.reg_i as usize + idx;
                    let data = self.reg_v[idx];
                    self.memory_write(target_pos, data);
                }
            }
            Instruction::Ldr { vr } => {
                for idx in 0..(vr + 1) {
                    self.reg_v[idx] = self.memory_read(self.reg_i as usize + idx)
                }
            }
            Instruction::Gdelay { vr } => {
                let amount = self.reg_delay_timer();
                self.reg_v[vr] = amount;
            }
            Instruction::Sdelay { vr } => {
                let amount = self.reg_v[vr];
                self.reg_delay_timer.store((amount + 1) as _, Ordering::SeqCst);
                let timer_clone = self.reg_delay_timer.clone();
                thread::spawn(move || loop {
                    let ticks_left = timer_clone.fetch_sub(1, Ordering::SeqCst) - 1;
                    if ticks_left > 0 {
                        println!("Still {} ticks left", ticks_left);
                        thread::sleep(time::Duration::from_millis(1000 / 60));
                    } else {
                        break;
                    }
                });
            }
        }
        self.pc += 2;
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn mem(&self) -> &Vec<u8> {
        &self.mem
    }

    pub fn reg_v(&self) -> &Vec<u8> {
        &self.reg_v
    }

    pub fn reg_i(&self) -> u16 {
        self.reg_i
    }

    pub fn reg_delay_timer(&self) -> u8 {
        self.reg_delay_timer.load(Ordering::SeqCst) as _
    }

    pub fn reg_sound_timer(&self) -> u8 {
        self.reg_sound_timer
    }

    pub fn stack(&self) -> &Vec<usize> {
        &self.stack
    }

    pub fn sp(&self) -> usize {
        self.sp
    }
}
