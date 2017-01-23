use std::convert::TryFrom;
use std::fmt;

pub enum Instruction {
    Cls,
    Ret,
    Jmp { addr: usize },
    Jsr { addr: usize },
    Skeq { vr: usize, k: u8 },
    Skne { vr: usize, k: u8 },
    Mov { vr: usize, k: u8 },
    Movr { vr: usize, vy: usize },
    And { vr: usize, vy: usize },
    Shr { vr: usize },
    Skner { vr: usize, vy: usize },
    Add { vr: usize, k: u8 },
    Addr { vr: usize, vy: usize },
    Subr { vr: usize, vy: usize },
    Mvi { k: u16 },
    Rnd { vr: usize, k: u8 },
    Sprite { rx: usize, ry: usize, s: usize },
    Skp { k: u8 },
    Sknp { k: u8 },
    Key { vr: usize },
    Sdelay { vr: usize },
    Gdelay { vr: usize },
    Adi { vr: usize },
    Font { vr: usize },
    Bcd { vr: usize },
    Str { vr: usize },
    Ldr { vr: usize },
}

impl TryFrom<u16> for Instruction {
    type Err = String;
    fn try_from(opcode: u16) -> Result<Self, Self::Err> {
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0x00E0 => Ok(Instruction::Cls),
                    0x00EE => Ok(Instruction::Ret),
                    _ => {
                        Err(format!("Opcode 0x{:x} not yet implemented (in 0x0000 branch)",
                                    opcode))
                    }
                }
            }
            0x1000 => k_op(opcode, |addr| Instruction::Jmp { addr: addr as usize }),
            0x2000 => k_op(opcode, |addr| Instruction::Jsr { addr: addr as usize }),
            0x3000 => vr_k_op(opcode, |vr, k| Instruction::Skeq { vr: vr, k: k }),
            0x4000 => vr_k_op(opcode, |vr, k| Instruction::Skne { vr: vr, k: k }),
            0x6000 => vr_k_op(opcode, |vr, k| Instruction::Mov { vr: vr, k: k }),
            0x7000 => vr_k_op(opcode, |vr, k| Instruction::Add { vr: vr, k: k }),
            0x8000 => {
                match opcode & 0xF00F {
                    0x8000 => vr_vy_op(opcode, |vr, vy| Instruction::Movr { vr: vr, vy: vy }),
                    0x8002 => vr_vy_op(opcode, |vr, vy| Instruction::And { vr: vr, vy: vy }),
                    0x8004 => vr_vy_op(opcode, |vr, vy| Instruction::Addr { vr: vr, vy: vy }),
                    0x8005 => vr_vy_op(opcode, |vr, vy| Instruction::Subr { vr: vr, vy: vy }),
                    0x8006 => vr_op(opcode, |vr| Instruction::Shr { vr: vr }),
                    _ => {
                        Err(format!("Opcode 0x{:x} not yet implemented (in 0x8000 branch)",
                                    opcode))
                    }
                }
            }
            0x9000 => vr_vy_op(opcode, |vr, vy| Instruction::Skner { vr: vr, vy: vy }),
            0xA000 => k_op(opcode, |k| Instruction::Mvi { k: k }),
            0xC000 => vr_k_op(opcode, |vr, k| Instruction::Rnd { vr: vr, k: k }),
            0xD000 => {
                let s = (opcode & 0x000F) as usize;
                let ry = ((opcode & 0x00F0) >> 4) as usize;
                let rx = ((opcode & 0x0F00) >> 8) as usize;
                Ok(Instruction::Sprite {
                    rx: rx,
                    ry: ry,
                    s: s,
                })
            }
            0xE000 => {
                match opcode & 0xF0FF {
                    0xE09E => vr_op(opcode, |k| Instruction::Skp { k: k as u8 }),
                    0xE0A1 => vr_op(opcode, |k| Instruction::Sknp { k: k as u8 }),
                    _ => {
                        Err(format!("Opcode 0x{:x} not yet implemented (in 0xE000 branch)",
                                    opcode))
                    }
                }
            }
            0xF000 => {
                match opcode & 0xF0FF {
                    0xF00A => vr_op(opcode, |vr| Instruction::Key { vr: vr }),
                    0xF007 => vr_op(opcode, |vr| Instruction::Gdelay { vr: vr }),
                    0xF015 => vr_op(opcode, |vr| Instruction::Sdelay { vr: vr }),
                    0xF01E => vr_op(opcode, |vr| Instruction::Adi { vr: vr }),
                    0xF029 => vr_op(opcode, |vr| Instruction::Font { vr: vr }),
                    0xF033 => vr_op(opcode, |vr| Instruction::Bcd { vr: vr }),
                    0xF055 => vr_op(opcode, |vr| Instruction::Str { vr: vr }),
                    0xF065 => vr_op(opcode, |vr| Instruction::Ldr { vr: vr }),
                    _ => {
                        Err(format!("Opcode 0x{:x} not yet implemented (in 0xF000 branch)",
                                    opcode))
                    }
                }
            }
            _ => Err(format!("Opcode 0x{:x} not yet implemented", opcode)),
        }
    }
}

fn vr_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(usize) -> Instruction
{
    let vr = ((opcode & 0x0F00) >> 8) as usize;
    Ok(f(vr))
}

fn vr_k_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(usize, u8) -> Instruction
{
    let k = (opcode & 0x00FF) as u8;
    let vr = ((opcode & 0xF00) >> 8) as usize;
    Ok(f(vr, k))
}

fn vr_vy_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(usize, usize) -> Instruction
{
    let vy = ((opcode & 0x00F0) >> 4) as usize;
    let vr = ((opcode & 0xF00) >> 8) as usize;
    Ok(f(vr, vy))
}

fn k_op<F>(opcode: u16, f: F) -> Result<Instruction, String>
    where F: FnOnce(u16) -> Instruction
{
    let k = (opcode & 0x0FFF) as u16;
    Ok(f(k))
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::Cls => write!(f, "cls"),
            Instruction::Ret => write!(f, "ret"),
            Instruction::Jmp { addr } => write!(f, "jmp    0x{:x}", addr),
            Instruction::Jsr { addr } => write!(f, "jsr    0x{:x}", addr),
            Instruction::Skeq { vr, k } => write!(f, "skeq   v{}, 0x{:x}", vr, k),
            Instruction::Skne { vr, k } => write!(f, "skne   v{}, 0x{:x}", vr, k),
            Instruction::Mov { vr, k } => write!(f, "mov    v{}, 0x{:x}", vr, k),
            Instruction::Movr { vr, vy } => write!(f, "mov    v{}, v{}", vr, vy),
            Instruction::And { vr, vy } => write!(f, "and    v{}, v{}", vr, vy),
            Instruction::Shr { vr } => write!(f, "shr    v{}", vr),
            Instruction::Skner { vr, vy } => write!(f, "skne   v{}, v{}", vr, vy),
            Instruction::Add { vr, k } => write!(f, "add    v{}, 0x{:x}", vr, k),
            Instruction::Addr { vr, vy } => write!(f, "add    v{}, v{}", vr, vy),
            Instruction::Subr { vr, vy } => write!(f, "sub    v{}, v{}", vr, vy),
            Instruction::Mvi { k } => write!(f, "mvi    0x{:x}", k),
            Instruction::Rnd { vr, k } => write!(f, "rnd    v{}, 0x{:x}", vr, k),
            Instruction::Sprite { rx, ry, s } => write!(f, "sprite {},{},{}", rx, ry, s),
            Instruction::Skp { k } => write!(f, "skp    0x{:x}", k),
            Instruction::Sknp { k } => write!(f, "sknp   0x{:x}", k),
            Instruction::Key { vr } => write!(f, "key    v{}", vr),
            Instruction::Sdelay { vr } => write!(f, "sdelay  v{}", vr),
            Instruction::Gdelay { vr } => write!(f, "gdelay  v{}", vr),
            Instruction::Adi { vr } => write!(f, "adi    v{}", vr),
            Instruction::Font { vr } => write!(f, "font   v{}", vr),
            Instruction::Bcd { vr } => write!(f, "bcd    v{}", vr),
            Instruction::Str { vr } => write!(f, "str    v0-v{}", vr),
            Instruction::Ldr { vr } => write!(f, "ldr    v0-v{}", vr),
        }
    }
}
