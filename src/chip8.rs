use std::fs::File;
use std::io::prelude::*;
use std::convert::TryFrom;
use super::instruction::Instruction;
use super::video_engine::VideoEngine;

const FONT_BASE_ADDR: usize = 0x0;
const NUM_FONTS: usize = 16;
const FONT_SIZE: usize = 5;
const FONT_MAP: [u8; NUM_FONTS * FONT_SIZE] =
    [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0,
     0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
     0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0,
     0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
     0xF0, 0x80, 0x80, 0x80, 0xF0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0xF0,
     0xF0, 0x80, 0xF0, 0x80, 0x80];

pub struct Chip8 {
    mem: Vec<u8>,
    reg_v: Vec<u8>,
    reg_i: u16,
    reg_delay_timer: u8,
    reg_sound_timer: u8,
    pc: usize,
    sp: usize,
    stack: Vec<usize>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            mem: vec![0; 0xFFF],
            reg_v: vec![0; 16],
            reg_i: 0,
            reg_delay_timer: 0,
            reg_sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: vec![0; 16],
        };
        chip8.load_fonts();
        chip8
    }

    fn load_fonts(&mut self) {
        for i in FONT_BASE_ADDR..FONT_MAP.len() {
            self.mem[i] = FONT_MAP[i - FONT_BASE_ADDR]
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut f: File = File::open(path).expect(&format!("Cannot open file {}", path));
        let mut buf: Vec<u8> = Vec::new();
        let read = f.read_to_end(&mut buf).expect(&format!("Cannot read from file {}", path));
        println!("Loaded {} bytes", read);
        for b in 0..buf.len() {
            self.mem[0x200 + b] = buf[b];
        }
    }

    pub fn step(&mut self, video_engine: &mut VideoEngine) {
        let hi_nibble = self.mem[self.pc] as u16;
        let lo_nibble = self.mem[self.pc + 1] as u16;
        let opcode = (hi_nibble << 8) | lo_nibble;
        match Instruction::try_from(opcode) {
            Err(msg) => panic!("Error decoding instruction at 0x{0:03x}: {1}", self.pc, msg),
            Ok(instruction) => self.step_instruction(instruction, video_engine),
        }
    }

    fn memory_read(&self, pos: usize) -> u8 {
        let val = self.mem[pos];
        // println!("[R][0x{:03x}] 0x{:03x}", pos, val);
        val
    }

    fn memory_write(&mut self, pos: usize, data: u8) {
        self.mem[pos] = data;
        // println!("[W][0x{:03x}] 0x{:03x}", pos, data);
    }

    fn step_instruction(&mut self, instruction: Instruction, video_engine: &mut VideoEngine) {
        match instruction {
            Instruction::Cls => video_engine.cls(),
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
            Instruction::Shr { vr } => {
                let current_val = self.reg_v[vr];
                self.reg_v[0xF] = current_val & 1;
                self.reg_v[vr] = current_val >> 1;
            }
            Instruction::Skeq { vr, k } => {
                if self.reg_v[vr] == k {
                    self.pc += 2
                }
            }
            Instruction::Add { vr, k } => {
                let old_val = self.reg_v[vr];
                self.reg_v[vr] = old_val.wrapping_add(k);
            }
            Instruction::Mvi { k } => self.reg_i = k,
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
                            let collision = video_engine.set_pixel_to_1(x + xline, y + yline);
                            if collision {
                                self.reg_v[0xF] = 1;
                            }
                        } else {
                            video_engine.set_pixel_to_0(x + xline, y + yline);
                        }

                    }
                }
            }
            Instruction::Key { vr } => {
                self.reg_v[vr] = video_engine.wait_for_key_input();
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
                self.mem[i + 0] = value / 100;
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
        }
        video_engine.draw();
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
        self.reg_delay_timer
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
